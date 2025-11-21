use clap::{Arg, Command};
use serde::{Deserialize, Serialize};
use std::fs;
use std::io::Write;
use std::path::PathBuf;
use std::time::{SystemTime, UNIX_EPOCH};
use reqwest::header::{AUTHORIZATION, CONTENT_TYPE};
use serde_json::Value;
use risc0_zkvm::Receipt;

#[derive(Serialize)]
struct RegisterRequest<'a> {
    email: &'a str,
    password: &'a str,
}

#[derive(Serialize)]
struct LoginRequest<'a> {
    email: &'a str,
    password: &'a str,
}

#[derive(Deserialize)]
struct TokenResponse {
    access_token: String,
    token_type: String,
    expires_in: u64,
}

#[derive(Serialize, Deserialize)]
struct AuthStore {
    access_token: String,
    token_type: String,
    expires_at: u64,
}

fn config_dir() -> PathBuf {
    if let Ok(dir) = std::env::var("VERSE_CONFIG_DIR") {
        return PathBuf::from(dir);
    }
    if let Ok(xdg) = std::env::var("XDG_CONFIG_HOME") {
        return PathBuf::from(xdg).join("verse");
    }
    let home = std::env::var("HOME").unwrap_or_else(|_| ".".to_string());
    PathBuf::from(home).join(".config").join("verse")
}

fn auth_path() -> PathBuf {
    config_dir().join("auth.json")
}

fn save_auth(auth: &AuthStore) -> std::io::Result<()> {
    let dir = config_dir();
    fs::create_dir_all(&dir)?;
    let path = auth_path();
    let json = serde_json::to_vec_pretty(auth).expect("serialize auth");
    let mut file = fs::File::create(&path)?;
    #[cfg(unix)]
    {
        use std::os::unix::fs::PermissionsExt;
        let mut perms = file.metadata()?.permissions();
        perms.set_mode(0o600);
        file.set_permissions(perms)?;
    }
    file.write_all(&json)?;
    Ok(())
}

fn load_auth() -> Result<AuthStore, String> {
    let path = auth_path();
    let data = fs::read_to_string(&path).map_err(|e| format!("Failed to read auth file ({}): {}", path.display(), e))?;
    let auth: AuthStore = serde_json::from_str(&data).map_err(|e| format!("Failed to parse auth file: {}", e))?;
    let now = SystemTime::now().duration_since(UNIX_EPOCH).unwrap_or_default().as_secs();
    if now >= auth.expires_at { return Err("Saved token has expired. Please run 'verse login' again.".into()); }
    Ok(auth)
}

// Load a CSV file into a 2D array (Vec<Vec<f32>>). Skips empty lines and optional header row.
fn load_csv_as_2d(path: &str) -> Result<Vec<Vec<f32>>, String> {
    let text = fs::read_to_string(path)
        .map_err(|e| format!("Failed to read CSV file ({}): {}", path, e))?;
    let mut rows: Vec<Vec<f32>> = Vec::new();
    for (i, line) in text.lines().enumerate() {
        let line = line.trim();
        if line.is_empty() { continue; }
        if i == 0 {
            // If first line has any non-float token, treat it as header and skip
            let toks: Vec<&str> = line.split(',').map(|s| s.trim()).collect();
            if toks.iter().any(|t| t.parse::<f32>().is_err()) {
                continue;
            }
        }
        let mut row: Vec<f32> = Vec::new();
        for tok in line.split(',').map(|s| s.trim()) {
            match tok.parse::<f32>() {
                Ok(v) => row.push(v),
                Err(e) => return Err(format!("Line {}: failed to parse '{}' as f32: {}", i + 1, tok, e)),
            }
        }
        if !row.is_empty() { rows.push(row); }
    }
    if rows.is_empty() { return Err("CSV contained no numeric rows".into()); }
    Ok(rows)
}

// Generate Rust code for get_dataset() from 2D rows (last column is label)
fn generate_get_dataset_code(rows: &[Vec<f32>]) -> Result<String, String> {
    if rows.is_empty() { return Err("No rows to generate dataset".into()); }
    let mut feat_len: Option<usize> = None;
    let mut items: Vec<String> = Vec::with_capacity(rows.len());
    for (i, r) in rows.iter().enumerate() {
        if r.len() < 2 { return Err(format!("Row {} has fewer than 2 columns", i + 1)); }
        let flen = r.len() - 1;
        if let Some(expected) = feat_len {
            if expected != flen { return Err(format!("Inconsistent columns at row {}: expected {} features before label, got {}", i + 1, expected, flen)); }
        } else {
            feat_len = Some(flen);
        }
        let (features, label) = r.split_at(flen);
        let feats_str = features
            .iter()
            .map(|v| {
                let s = v.to_string();
                if s.contains('.') { s } else { format!("{}.0", s) }
            })
            .collect::<Vec<_>>()
            .join(", ");
        let lbl_s = {
            let s = label[0].to_string();
            if s.contains('.') { s } else { format!("{}.0", s) }
        };
        items.push(format!("        (vec![{}], {}),", feats_str, lbl_s));
    }
    let body = items.join("\n");
    let code = format!(
        "fn get_dataset() -> Vec<(Vec<f32>, f32)> {{\n    vec![\n{}\n    ]\n}}\n",
        body
    );
    Ok(code)
}

// Replace the get_dataset() function implementation in a template file.
fn write_dataset_to_template(template_path: &str, rows: &[Vec<f32>]) -> Result<(), String> {
    let mut content = fs::read_to_string(template_path)
        .map_err(|e| format!("Failed to read template ({}): {}", template_path, e))?;
    let new_fn = generate_get_dataset_code(rows)?;

    // Find existing get_dataset signature line index boundaries and replace up to the next lone '}'
    let lines: Vec<&str> = content.lines().collect();
    let mut start_idx: Option<usize> = None;
    for (i, line) in lines.iter().enumerate() {
        if line.trim_start().starts_with("fn get_dataset() -> Vec<(Vec<f32>, f32)>") {
            start_idx = Some(i);
            break;
        }
    }
    let start = start_idx.ok_or("Failed to find get_dataset() in template")?;
    // Find the closing brace line for this function
    let mut end = None;
    for i in start+1..lines.len() {
        if lines[i].trim() == "}" {
            end = Some(i);
            break;
        }
    }
    let end = end.ok_or("Failed to find end of get_dataset() in template")?;

    // Rebuild file: lines before start, then new_fn, then lines after end
    let before = lines[..start].join("\n");
    let after = if end + 1 < lines.len() { lines[end+1..].join("\n") } else { String::new() };
    let mut new_content = String::new();
    if !before.is_empty() { new_content.push_str(&before); new_content.push('\n'); }
    new_content.push_str(&new_fn);
    if !after.is_empty() { new_content.push_str(&after); new_content.push('\n'); }

    fs::write(template_path, new_content)
        .map_err(|e| format!("Failed to write template ({}): {}", template_path, e))?
        ;
    Ok(())
}

// Copy the updated template into the actual guest code location.
// Expects guest_dir to be the workspace root of the guest (containing `methods/guest/src/main.rs`).
fn copy_template_to_guest(template_path: &str, guest_dir: &str) -> Result<PathBuf, String> {
    let content = fs::read_to_string(template_path)
        .map_err(|e| format!("Failed to read template ({}): {}", template_path, e))?;
    let guest_main = PathBuf::from(guest_dir).join("methods/guest/src/main.rs");
    if !guest_main.exists() {
        return Err(format!(
            "Guest main not found at {} (dir was '{}'). Ensure the path is correct.",
            guest_main.display(), guest_dir
        ));
    }
    fs::write(&guest_main, content)
        .map_err(|e| format!("Failed to write guest main ({}): {}", guest_main.display(), e))?;
    Ok(guest_main)
}

fn pretty_print_models(body: &str) {
    match serde_json::from_str::<Value>(body) {
        Ok(Value::Array(items)) => {
            if items.is_empty() {
                println!("No models found.");
                return;
            }
            println!("Models ({}):", items.len());
            for (i, item) in items.iter().enumerate() {
                let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("-");
                let name = item.get("name").and_then(|v| v.as_str()).unwrap_or("-");
                let vector = item.get("vectorFormat").and_then(|v| v.as_str()).unwrap_or("-");
                let created = item.get("createdAt").and_then(|v| v.as_str()).unwrap_or("-");
                let updated = item.get("updatedAt").and_then(|v| v.as_str()).unwrap_or("-");
                println!("\n{}. {}", i + 1, name);
                println!("   id:           {}", id);
                println!("   vectorFormat: {}", vector);
                println!("   createdAt:    {}", created);
                println!("   updatedAt:    {}", updated);
            }
        }
        _ => {
            // Fallback to raw output if parsing fails or response isn't an array
            println!("{}", body);
        }
    }
}

fn pretty_print_validation_request(body: &str) {
    match serde_json::from_str::<Value>(body) {
        Ok(Value::Object(obj)) => {
            let id = obj.get("id").and_then(|v| v.as_str()).unwrap_or("-");
            let model_id = obj.get("modelId").and_then(|v| v.as_str()).unwrap_or("-");
            let verifier_id = obj.get("verifierId").and_then(|v| v.as_str()).unwrap_or("-");
            let elf_url = obj.get("elfFileUrl").and_then(|v| v.as_str()).unwrap_or("-");
            let json_url = obj.get("jsonUrl").and_then(|v| v.as_str()).unwrap_or("-");
            let status = obj.get("status").and_then(|v| v.as_str()).unwrap_or("-");
            let created = obj.get("createdAt").and_then(|v| v.as_str()).unwrap_or("-");

            let proof_hash = match obj.get("proofHash") {
                Some(Value::Array(arr)) => {
                    let nums: Vec<String> = arr
                        .iter()
                        .map(|x| match x {
                            Value::Number(n) => n.to_string(),
                            other => other.to_string(),
                        })
                        .collect();
                    format!("[{}]", nums.join(", "))
                }
                Some(other) => other.to_string(),
                None => "-".to_string(),
            };

            println!("Validation request submitted:\n");
            println!("  id:           {}", id);
            println!("  modelId:      {}", model_id);
            println!("  verifierId:   {}", verifier_id);
            println!("  elfFileUrl:   {}", elf_url);
            println!("  jsonUrl:      {}", json_url);
            println!("  proofHash:    {}", proof_hash);
            println!("  status:       {}", status);
            println!("  createdAt:    {}", created);
        }
        _ => println!("{}", body),
    }
}

fn pretty_print_pending_validations(body: &str) {
    match serde_json::from_str::<Value>(body) {
        Ok(Value::Object(obj)) => {
            let models = obj.get("models").and_then(|v| v.as_array());
            if models.is_none() {
                println!("{}", body);
                return;
            }
            let models = models.unwrap();
            let mut total_pending = 0usize;
            let mut any = false;
            for (mi, m) in models.iter().enumerate() {
                let model_id = m.get("id").and_then(|v| v.as_str()).unwrap_or("-");
                let model_name = m.get("name").and_then(|v| v.as_str()).unwrap_or("");
                // Own the array to avoid borrowing a temporary
                let vrs: Vec<Value> = m
                    .get("validationRequests")
                    .and_then(|v| v.as_array())
                    .cloned()
                    .unwrap_or_default();
                let pending: Vec<&Value> = vrs
                    .iter()
                    .filter(|vr| vr.get("status").and_then(|s| s.as_str()) == Some("pending"))
                    .collect();
                if pending.is_empty() {
                    continue;
                }
                any = true;
                println!(
                    "Model {} (id: {}{}):",
                    mi + 1,
                    model_id,
                    if model_name.is_empty() { String::new() } else { format!(", name: {}", model_name) }
                );
                for (i, vr) in pending.iter().enumerate() {
                    let id = vr.get("id").and_then(|v| v.as_str()).unwrap_or("-");
                    let verifier_id = vr.get("verifierId").and_then(|v| v.as_str()).unwrap_or("-");
                    let elf_url = vr.get("elfFileUrl").and_then(|v| v.as_str()).unwrap_or("-");
                    let created = vr.get("createdAt").and_then(|v| v.as_str()).unwrap_or("-");
                    println!("  {}. validation:", i + 1);
                    println!("     id:         {}", id);
                    println!("     verifierId: {}", verifier_id);
                    println!("     elfFileUrl: {}", elf_url);
                    println!("     status:     pending");
                    println!("     createdAt:  {}", created);
                }
                total_pending += pending.len();
                println!("");
            }
            if !any {
                println!("No pending validation requests found.");
            } else {
                println!("Total pending: {}", total_pending);
            }
        }
        _ => println!("{}", body),
    }
}

fn pretty_print_verifier_requests(body: &str) {
    match serde_json::from_str::<Value>(body) {
        Ok(Value::Array(items)) => {
            if items.is_empty() {
                println!("No validation requests found.");
                return;
            }
            println!("Your validation requests ({}):", items.len());
            for (i, item) in items.iter().enumerate() {
                let id = item.get("id").and_then(|v| v.as_str()).unwrap_or("-");
                let model_id = item.get("modelId").and_then(|v| v.as_str()).unwrap_or("-");
                let verifier_id = item.get("verifierId").and_then(|v| v.as_str()).unwrap_or("-");
                let elf_url = item.get("elfFileUrl").and_then(|v| v.as_str()).unwrap_or("-");
                let json_url = item.get("jsonUrl").and_then(|v| v.as_str()).unwrap_or("-");
                let status = item.get("status").and_then(|v| v.as_str()).unwrap_or("-");
                let created = item.get("createdAt").and_then(|v| v.as_str()).unwrap_or("-");
                let model_name = item
                    .get("model")
                    .and_then(|m| m.get("name"))
                    .and_then(|v| v.as_str())
                    .unwrap_or("");
                let proof_hash = item.get("proofHash").and_then(|v| v.as_str()).unwrap_or("-");

                println!("\n{}. validation request:", i + 1);
                println!("   id:           {}", id);
                println!("   modelId:      {}{}",
                    model_id,
                    if model_name.is_empty() { String::new() } else { format!(", name: {}", model_name) }
                );
                println!("   verifierId:   {}", verifier_id);
                println!("   elfFileUrl:   {}", elf_url);
                println!("   jsonUrl:      {}", json_url);
                println!("   proofHash:    {}", proof_hash);
                println!("   status:       {}", status);
                println!("   createdAt:    {}", created);
            }
        }
        _ => println!("{}", body),
    }
}

fn main() {
    let matches = Command::new("verse")
        .version("1.0")
        .author("Salai Kowshikan")
        .about("This is V.E.R.S.E, a command line tool to provide a model validation interface that protects the privacy of both the parties involved.")
        .arg(
            Arg::new("name")
                .short('n')
                .long("name")
                .help("Sets your name")
                .value_name("NAME"),
        )
        .subcommand(
            Command::new("request")
                .about("Build the ZK guest and send a validation request for a model")
                .arg(
                    Arg::new("list")
                        .long("list")
                        .help("List all validation requests placed by you (verifier)")
                        .required(false)
                        .action(clap::ArgAction::SetTrue)
                        .conflicts_with_all(["model-id", "dir", "elf"]),
                )
                .arg(
                    Arg::new("model-id")
                        .long("model-id")
                        .short('m')
                        .help("The target model ID to request validation for")
                        .value_name("MODEL_ID")
                        .required_unless_present("list"),
                )
                .arg(
                    Arg::new("dir")
                        .short('d')
                        .long("dir")
                        .help("Path to the ZK-guest workspace directory")
                        .value_name("PATH")
                        .default_value("../ZK-guest"),
                )
                .arg(
                    Arg::new("elf")
                        .long("elf")
                        .help("Path to an already exported ELF file (skips autodetect)")
                        .value_name("FILE")
                        .required(false),
                )
                .arg(
                    Arg::new("dataset")
                        .long("dataset")
                        .help("Path to a dataset CSV file to load as a 2D array")
                        .value_name("CSV_PATH")
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("register")
                .about("Register a new user on V.E.R.S.E")
                .arg(
                    Arg::new("email")
                        .help("Email address to register")
                        .value_name("EMAIL")
                        .required(true),
                )
                .arg(
                    Arg::new("password")
                        .help("Password for the account")
                        .value_name("PASSWORD")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("login")
                .about("Log into V.E.R.S.E")
                .arg(
                    Arg::new("email")
                        .help("Email address")
                        .value_name("EMAIL")
                        .required(true),
                )
                .arg(
                    Arg::new("password")
                        .help("Password")
                        .value_name("PASSWORD")
                        .required(true),
                ),
        )
        .subcommand(
            Command::new("prove")
                .about("Download the ELF for a validation request and run the Zk-host prover")
                .arg(
                    Arg::new("model-id")
                        .long("model-id")
                        .short('m')
                        .help("The model ID associated with the request (for info only)")
                        .value_name("MODEL_ID")
                        .required(true),
                )
                .arg(
                    Arg::new("request-id")
                        .long("request-id")
                        .short('r')
                        .help("The validation request ID to prove")
                        .value_name("REQUEST_ID")
                        .required(true),
                )
                .arg(
                    Arg::new("zk-host-dir")
                        .long("zk-host-dir")
                        .help("Path to the Zk-host workspace directory where guest-elf will be saved")
                        .value_name("DIR")
                        .default_value("../Zk-host"),
                )
                .arg(
                    Arg::new("dataset")
                        .long("dataset")
                        .help("Path to a dataset CSV file to include with the request")
                        .value_name("CSV_PATH")
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("verify")
                .about("Download the proof JSON for a validation request and verify it using RISC Zero")
                .arg(
                    Arg::new("request-id")
                        .long("request-id")
                        .short('r')
                        .help("The validation request ID whose proof to verify")
                        .value_name("REQUEST_ID")
                        .required(true),
                )
                .arg(
                    Arg::new("out")
                        .long("out")
                        .help("Optional path to save downloaded proof.json; defaults to ./proof.json")
                        .value_name("FILE")
                        .required(false),
                ),
        )
        .subcommand(
            Command::new("model")
                .about("Manage your models")
                .arg(
                    Arg::new("requests")
                        .long("requests")
                        .help("List pending validation requests for your models")
                        .required(false)
                        .action(clap::ArgAction::SetTrue),
                )
                .subcommand(
                    Command::new("list")
                        .about("List all of your models"),
                )
                .subcommand(
                    Command::new("new")
                        .about("Create a new model")
                        .long_about(
                            "Create a new model for the authenticated user.\n\nFields:\n  - vectorFormat: The size/order of the input vector and which feature goes at each index.\n  - name: A short name for the model (e.g., Skin cancer prediction).\n  - description: More about how the model was trained and what its predictions mean.\n\nExample JSON body that the API receives:\n{\n  \"vectorFormat\": \"len=3; x[0]=age, x[1]=bmi, x[2]=bp\",\n  \"name\": \"Skin cancer prediction\",\n  \"description\": \"Trained on dermatoscopic images; outputs malignancy probability.\"\n}"
                        )
                        .after_help(
                            "Tip: ensure you are logged in (verse login) so your JWT is available for Authorization."
                        )
                        .arg(
                            Arg::new("vector-format")
                                .long("vector-format")
                                .help("Vector format description, e.g. feature index mapping")
                                .value_name("FORMAT")
                                .required(true),
                        )
                        .arg(
                            Arg::new("name")
                                .long("name")
                                .help("Name of the model (e.g., Skin cancer prediction)")
                                .value_name("NAME")
                                .required(true),
                        )
                        .arg(
                            Arg::new("description")
                                .long("description")
                                .help("Description: how the model was trained and what its predictions mean")
                                .value_name("TEXT")
                                .required(false),
                        ),
                ),
        )
        .get_matches();

    match matches.subcommand() {
        Some(("request", sub_m)) => {
            if sub_m.get_flag("list") {
                let auth = match load_auth() { Ok(a) => a, Err(e) => { eprintln!("{}", e); std::process::exit(1); } };
                let url = std::env::var("VERSE_API_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
                let endpoint = format!("{}/api/model/validation-requests/verifier", url.trim_end_matches('/'));
                let client = reqwest::blocking::Client::new();
                match client
                    .get(endpoint)
                    .header(AUTHORIZATION, format!("Bearer {}", auth.access_token))
                    .send()
                {
                    Ok(resp) => {
                        let status = resp.status();
                        match resp.text() {
                            Ok(body) => {
                                if status.is_success() { pretty_print_verifier_requests(&body); std::process::exit(0); }
                                else { eprintln!("List failed ({}): {}", status, body); std::process::exit(1); }
                            }
                            Err(e) => { eprintln!("Failed to read response body: {}", e); std::process::exit(1); }
                        }
                    }
                    Err(e) => { eprintln!("HTTP request error: {}", e); std::process::exit(1); }
                }
            }

            let model_id = sub_m
                .get_one::<String>("model-id")
                .map(String::as_str)
                .expect("--model-id is required");

            let dir = sub_m
                .get_one::<String>("dir")
                .map(String::as_str)
                .unwrap_or("ZK-guest");

            let explicit_elf = sub_m.get_one::<String>("elf").map(String::as_str);
            let dataset_path = sub_m.get_one::<String>("dataset").map(String::as_str);

            // If dataset path provided, load it now as 2D array
            if let Some(csv_path) = dataset_path {
                match load_csv_as_2d(csv_path) {
                    Ok(rows) => {
                        let cols = rows.get(0).map(|r| r.len()).unwrap_or(0);
                        println!(
                            "Loaded dataset: {} rows x {} cols from {}",
                            rows.len(),
                            cols,
                            csv_path
                        );
                        // Optional preview
                        let preview = rows.iter().take(3);
                        for (i, r) in preview.enumerate() {
                            println!("  row {:>3}: {:?}", i, r);
                        }
                        // Write into template get_dataset() so guest can embed the dataset
                        let template_path = {
                            let base = env!("CARGO_MANIFEST_DIR");
                            format!("{}/template.txt", base)
                        };
                        match write_dataset_to_template(&template_path, &rows) {
                            Ok(_) => println!("Updated {} with get_dataset() from the loaded CSV.", template_path),
                            Err(e) => { eprintln!("Failed to update template: {}", e); std::process::exit(1); }
                        }
                        // Copy the updated template into the guest code so it will be used on build/run
                        match copy_template_to_guest(&template_path, dir) {
                            Ok(guest_main) => println!("Copied template into guest: {}", guest_main.display()),
                            Err(e) => { eprintln!("Failed to copy template into guest: {}", e); std::process::exit(1); }
                        }
                    }
                    Err(e) => {
                        eprintln!("Failed to load dataset CSV: {}", e);
                        std::process::exit(1);
                    }
                }
            }

            if explicit_elf.is_none() {
                println!("Running `cargo run --release` in: {}", dir);
                let status = std::process::Command::new("cargo")
                    .arg("run")
                    .arg("--release")
                    .current_dir(dir)
                    .status();
                match status {
                    Ok(s) => {
                        if !s.success() {
                            if let Some(code) = s.code() { eprintln!("Guest run failed with exit code {}", code); }
                            else { eprintln!("Guest run terminated by signal"); }
                            std::process::exit(1);
                        }
                    }
                    Err(e) => { eprintln!("Failed to execute cargo: {}", e); std::process::exit(1); }
                }
            }
            let elf_path: PathBuf = if let Some(p) = explicit_elf {
                PathBuf::from(p)
            } else {
                PathBuf::from("../ZK-guest/LinearRegression_exported")
            };

            if !elf_path.exists() { eprintln!("ELF file not found: {}", elf_path.display()); std::process::exit(1); }

            let id_path = PathBuf::from(dir).join("LinearRegression_ID_exported");
            let hash_value = match fs::read_to_string(&id_path) {
                Ok(s) => s.trim().to_string(),
                Err(e) => {
                    eprintln!(
                        "Failed to read hash value from {}: {}. Ensure the guest wrote 'LinearRegression_ID_exported'.",
                        id_path.display(), e
                    );
                    std::process::exit(1);
                }
            };

            // let auth = match load_auth() {
            //     Ok(a) => a,
            //     Err(e) => { eprintln!("{}", e); std::process::exit(1); }
            // };
            // let url = std::env::var("VERSE_API_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
            // let endpoint = format!("{}/api/model/validation-request", url.trim_end_matches('/'));
            // println!("Uploading validation request for model {} with ELF: {}", model_id, elf_path.display());

            // let client = reqwest::blocking::Client::new();
            // let file_name = elf_path.file_name().and_then(|s| s.to_str()).unwrap_or("guest.elf");
            // let file = match fs::File::open(&elf_path) {
            //     Ok(f) => f,
            //     Err(e) => { eprintln!("Failed to open ELF file: {}", e); std::process::exit(1); }
            // };

            // let part = reqwest::blocking::multipart::Part::reader(file)
            //     .file_name(file_name.to_string())
            //     .mime_str("application/octet-stream").unwrap();

            // let form = reqwest::blocking::multipart::Form::new()
            //     .text("model_id", model_id.to_string())
            //     .text("hashValue", hash_value)
            //     .part("elf_file", part);

            // match client.post(endpoint)
            //     .header(AUTHORIZATION, format!("Bearer {}", auth.access_token))
            //     .multipart(form)
            //     .send() {
            //     Ok(resp) => {
            //         let status = resp.status();
            //         match resp.text() {
            //             Ok(body) => {
            //                 if status.is_success() { pretty_print_validation_request(&body); std::process::exit(0); }
            //                 else { eprintln!("Request failed ({}): {}", status, body); std::process::exit(1); }
            //             }
            //             Err(e) => { eprintln!("Failed to read response body: {}", e); std::process::exit(1); }
            //         }
            //     }
            //     Err(e) => { eprintln!("HTTP request error: {}", e); std::process::exit(1); }
            // }
        }
        Some(("register", sub_m)) => {
            let email = sub_m
                .get_one::<String>("email")
                .map(String::as_str)
                .expect("email is required");
            let password = sub_m
                .get_one::<String>("password")
                .map(String::as_str)
                .expect("password is required");

            let url = std::env::var("VERSE_API_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
            let endpoint = format!("{}/api/users/register", url.trim_end_matches('/'));

            let payload = RegisterRequest { email, password };

            println!("Registering '{}' at {}...", email, endpoint);

            let client = reqwest::blocking::Client::new();
            match client.post(endpoint).json(&payload).send() {
                Ok(resp) => {
                    let status = resp.status();
                    match resp.text() {
                        Ok(body) => {
                            if status.is_success() {
                                println!("Success: {}", body);
                                std::process::exit(0);
                            } else {
                                eprintln!("Registration failed ({}): {}", status, body);
                                std::process::exit(1);
                            }
                        }
                        Err(e) => {
                            eprintln!("Failed to read response body: {}", e);
                            std::process::exit(1);
                        }
                    }
                }
                Err(e) => {
                    eprintln!("HTTP request error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(("login", sub_m)) => {
            let email = sub_m
                .get_one::<String>("email")
                .map(String::as_str)
                .expect("email is required");
            let password = sub_m
                .get_one::<String>("password")
                .map(String::as_str)
                .expect("password is required");

            let url = std::env::var("VERSE_API_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
            let endpoint = format!("{}/api/users/login", url.trim_end_matches('/'));

            let payload = LoginRequest { email, password };

            println!("Logging in '{}' at {}...", email, endpoint);

            let client = reqwest::blocking::Client::new();
            match client.post(endpoint).json(&payload).send() {
                Ok(resp) => {
                    let status = resp.status();
                    if status.is_success() {
                        match resp.json::<TokenResponse>() {
                            Ok(token) => {
                                let now = SystemTime::now()
                                    .duration_since(UNIX_EPOCH)
                                    .unwrap_or_default()
                                    .as_secs();
                                let expires_at = now + token.expires_in.saturating_sub(30);
                                let store = AuthStore {
                                    access_token: token.access_token,
                                    token_type: token.token_type,
                                    expires_at,
                                };
                                if let Err(e) = save_auth(&store) {
                                    eprintln!("Login succeeded but failed to save token: {}", e);
                                    std::process::exit(1);
                                }
                                println!(
                                    "Login successful. Token saved to {}",
                                    auth_path().display()
                                );
                                std::process::exit(0);
                            }
                            Err(e) => {
                                eprintln!("Failed to parse token response: {}", e);
                                std::process::exit(1);
                            }
                        }
                    } else {
                        match resp.text() {
                            Ok(body) => {
                                eprintln!("Login failed ({}): {}", status, body);
                            }
                            Err(e) => eprintln!("Login failed ({}), and couldn't read body: {}", status, e),
                        }
                        std::process::exit(1);
                    }
                }
                Err(e) => {
                    eprintln!("HTTP request error: {}", e);
                    std::process::exit(1);
                }
            }
        }
        Some(("prove", sub_m)) => {
            let _model_id = sub_m
                .get_one::<String>("model-id")
                .map(String::as_str)
                .expect("--model-id is required");
            let request_id = sub_m
                .get_one::<String>("request-id")
                .map(String::as_str)
                .expect("--request-id is required");
            let zk_host_dir = sub_m
                .get_one::<String>("zk-host-dir")
                .map(String::as_str)
                .unwrap_or("../Zk-host");

            let auth = match load_auth() { Ok(a) => a, Err(e) => { eprintln!("{}", e); std::process::exit(1); } };
            let url = std::env::var("VERSE_API_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
            let base = url.trim_end_matches('/');

            let info_endpoint = format!("{}/api/model/validation-request/{}", base, request_id);
            let client = reqwest::blocking::Client::new();
            let info_body = match client
                .get(&info_endpoint)
                .header(AUTHORIZATION, format!("Bearer {}", auth.access_token))
                .send()
            {
                Ok(resp) => {
                    let status = resp.status();
                    match resp.text() {
                        Ok(body) => {
                            if !status.is_success() { eprintln!("Fetch request info failed ({}): {}", status, body); std::process::exit(1); }
                            body
                        }
                        Err(e) => { eprintln!("Failed to read response body: {}", e); std::process::exit(1); }
                    }
                }
                Err(e) => { eprintln!("HTTP request error: {}", e); std::process::exit(1); }
            };

            let info_json: Value = match serde_json::from_str(&info_body) {
                Ok(v) => v,
                Err(e) => { eprintln!("Failed to parse request info JSON: {}", e); std::process::exit(1); }
            };
            let elf_key_or_url = info_json.get("elfFileUrl").and_then(|v| v.as_str()).unwrap_or("");
            if elf_key_or_url.is_empty() { eprintln!("Request has no elfFileUrl"); std::process::exit(1); }
            let save_dir = PathBuf::from(zk_host_dir);
            if let Err(e) = fs::create_dir_all(&save_dir) { eprintln!("Failed to create dir {}: {}", save_dir.display(), e); std::process::exit(1); }
            let save_path = save_dir.join("guest-elf");

            println!("Prove: request {} => elfFileUrl: {}", request_id, elf_key_or_url);
            // Prefer public R2 bucket for relative keys; allow override via VERSE_R2_PUBLIC_URL
            let (download_url, use_public_bucket) = if elf_key_or_url.starts_with("http://") || elf_key_or_url.starts_with("https://") {
                (elf_key_or_url.to_string(), false)
            } else {
                let r2_base = std::env::var("VERSE_R2_PUBLIC_URL")
                    .unwrap_or_else(|_| "https://pub-eb24a8604ce54e00991962507f2d1cbb.r2.dev".to_string());
                (
                    format!("{}/{}", r2_base.trim_end_matches('/'), elf_key_or_url.trim_start_matches('/')),
                    true,
                )
            };

            println!("Downloading ELF from: {}", download_url);
            let mut req = client.get(&download_url);
            if !use_public_bucket {
                req = req.header(AUTHORIZATION, format!("Bearer {}", auth.access_token));
            }
            let mut resp = match req.send() {
                Ok(r) => r,
                Err(e) => { eprintln!("Download request error: {}", e); std::process::exit(1); }
            };
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().unwrap_or_default();
                if use_public_bucket {
                    eprintln!(
                        "Failed to download ELF ({}): {}\nTried public bucket URL: {}\nHint: ensure VERSE_R2_PUBLIC_URL is correct or elfFileUrl points to the right object path.",
                        status, body, download_url
                    );
                } else {
                    eprintln!(
                        "Failed to download ELF ({}): {}\nHint: the server should serve '{}' at the API base; otherwise expose a download endpoint or return a full URL.",
                        status, body, elf_key_or_url
                    );
                }
                std::process::exit(1);
            }
            let mut out = match fs::File::create(&save_path) { Ok(f) => f, Err(e) => { eprintln!("Failed to create {}: {}", save_path.display(), e); std::process::exit(1); } };
            if let Err(e) = std::io::copy(&mut resp, &mut out) { eprintln!("Failed to save ELF to {}: {}", save_path.display(), e); std::process::exit(1); }
            println!("Saved ELF to {}", save_path.display());

            println!("Starting prover in {}. When prompted for 'Enter path to guest ELF file:', type: guest-elf", save_dir.display());
            let status = std::process::Command::new("cargo")
                .arg("run")
                .arg("--release")
                .current_dir(&save_dir)
                .status();
            match status {
                Ok(s) => {
                    if !s.success() {
                        if let Some(code) = s.code() { eprintln!("Prover exited with code {}", code); }
                        else { eprintln!("Prover terminated by signal"); }
                        std::process::exit(1);
                    }
                }
                Err(e) => { eprintln!("Failed to start prover: {}", e); std::process::exit(1); }
            }

            let proof_path = save_dir.join("proof.json");
            if !proof_path.exists() {
                eprintln!("Expected proof file not found at {}", proof_path.display());
                std::process::exit(1);
            }
            println!("Uploading proof from {}...", proof_path.display());

            let put_endpoint = format!("{}/api/model/proof/{}", base, request_id);
            let proof_file = match fs::File::open(&proof_path) {
                Ok(f) => f,
                Err(e) => { eprintln!("Failed to open proof file: {}", e); std::process::exit(1); }
            };
            let proof_part = reqwest::blocking::multipart::Part::reader(proof_file)
                .file_name("proof.json".to_string())
                .mime_str("application/json").unwrap();
            let form = reqwest::blocking::multipart::Form::new()
                .part("json_file", proof_part);

            match client
                .put(&put_endpoint)
                .header(AUTHORIZATION, format!("Bearer {}", auth.access_token))
                .multipart(form)
                .send()
            {
                Ok(resp) => {
                    let status = resp.status();
                    match resp.text() {
                        Ok(body) => {
                            if status.is_success() { println!("Proof uploaded successfully: {}", body); std::process::exit(0); }
                            else { eprintln!("Proof upload failed ({}): {}", status, body); std::process::exit(1); }
                        }
                        Err(e) => { eprintln!("Failed to read response body: {}", e); std::process::exit(1); }
                    }
                }
                Err(e) => { eprintln!("HTTP request error during proof upload: {}", e); std::process::exit(1); }
            }
        }
        Some(("verify", sub_m)) => {
            let request_id = sub_m
                .get_one::<String>("request-id")
                .map(String::as_str)
                .expect("--request-id is required");
            let out_path = sub_m
                .get_one::<String>("out")
                .map(String::as_str)
                .unwrap_or("proof.json");

            let auth = match load_auth() { Ok(a) => a, Err(e) => { eprintln!("{}", e); std::process::exit(1); } };
            let url = std::env::var("VERSE_API_URL").unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
            let base = url.trim_end_matches('/');
            let info_endpoint = format!("{}/api/model/validation-request/{}", base, request_id);
            let client = reqwest::blocking::Client::new();

            // Fetch request info to get jsonUrl and proofHash
            let info_body = match client
                .get(&info_endpoint)
                .header(AUTHORIZATION, format!("Bearer {}", auth.access_token))
                .send()
            {
                Ok(resp) => {
                    let status = resp.status();
                    match resp.text() {
                        Ok(body) => { if !status.is_success() { eprintln!("Fetch request info failed ({}): {}", status, body); std::process::exit(1); } body }
                        Err(e) => { eprintln!("Failed to read response body: {}", e); std::process::exit(1); }
                    }
                }
                Err(e) => { eprintln!("HTTP request error: {}", e); std::process::exit(1); }
            };
            let info_json: Value = match serde_json::from_str(&info_body) { Ok(v) => v, Err(e) => { eprintln!("Failed to parse request info JSON: {}", e); std::process::exit(1); } };
            let json_key_or_url = info_json.get("jsonUrl").and_then(|v| v.as_str()).unwrap_or("");
            if json_key_or_url.is_empty() { eprintln!("Request has no jsonUrl (proof not available yet?)"); std::process::exit(1); }

            // Build public bucket URL for relative keys
            let download_url = if json_key_or_url.starts_with("http://") || json_key_or_url.starts_with("https://") {
                json_key_or_url.to_string()
            } else {
                let r2_base = std::env::var("VERSE_R2_PUBLIC_URL").unwrap_or_else(|_| "https://pub-eb24a8604ce54e00991962507f2d1cbb.r2.dev".to_string());
                format!("{}/{}", r2_base.trim_end_matches('/'), json_key_or_url.trim_start_matches('/'))
            };

            println!("Downloading proof from: {}", download_url);
            let mut resp = match client.get(&download_url).send() { Ok(r) => r, Err(e) => { eprintln!("Download request error: {}", e); std::process::exit(1); } };
            if !resp.status().is_success() {
                let status = resp.status();
                let body = resp.text().unwrap_or_default();
                eprintln!("Failed to download proof ({}): {}", status, body);
                std::process::exit(1);
            }
            let mut out = match fs::File::create(out_path) { Ok(f) => f, Err(e) => { eprintln!("Failed to create output file: {}", e); std::process::exit(1); } };
            if let Err(e) = std::io::copy(&mut resp, &mut out) { eprintln!("Failed to save proof: {}", e); std::process::exit(1); }
            println!("Saved proof to {}", out_path);

            // Parse proofHash into [u32; 8]
            let method_id_arr: [u32; 8] = {
                // proofHash is provided as a string HASH_ID; expect a comma-separated list or JSON array-like string
                let ph_val = info_json.get("proofHash").cloned().unwrap_or(Value::Null);
                let parse_err = || {
                    eprintln!("Invalid or missing proofHash in response; expected a comma-separated 8 u32 values or JSON array string.");
                    std::process::exit(1);
                };
                let mut nums: Vec<u32> = Vec::new();
                match ph_val {
                    Value::String(s) => {
                        // Accept formats like: "[1,2,3,4,5,6,7,8]" or "1,2,3,4,5,6,7,8"
                        let s = s.trim().trim_matches(|c| c == '[' || c == ']');
                        for part in s.split(',') {
                            let p = part.trim();
                            if p.is_empty() { continue; }
                            match p.parse::<u32>() { Ok(n) => nums.push(n), Err(_) => parse_err() }
                        }
                    }
                    Value::Array(arr) => {
                        for v in arr { match v.as_u64() { Some(n) if n <= u32::MAX as u64 => nums.push(n as u32), _ => parse_err() } }
                    }
                    _ => parse_err(),
                }
                if nums.len() != 8 { parse_err(); }
                [nums[0], nums[1], nums[2], nums[3], nums[4], nums[5], nums[6], nums[7]]
            };

            // Deserialize receipt from saved proof
            let data = match fs::read_to_string(out_path) { Ok(s) => s, Err(e) => { eprintln!("Failed to read proof file: {}", e); std::process::exit(1); } };
            let receipt: Receipt = match serde_json::from_str(&data) { Ok(r) => r, Err(e) => { eprintln!("Failed to parse receipt JSON: {}", e); std::process::exit(1); } };

            // Verify
            match receipt.verify(method_id_arr) {
                Ok(_) => { println!("✅ Proof verified successfully!"); std::process::exit(0); }
                Err(e) => { println!("❌ Verification failed: {:?}", e); std::process::exit(1); }
            }
        }
        Some(("model", sub_m)) => {
            let url = std::env::var("VERSE_API_URL")
                .unwrap_or_else(|_| "http://127.0.0.1:8000".to_string());
            let base = url.trim_end_matches('/');
            let client = reqwest::blocking::Client::new();

            if sub_m.get_flag("requests") {
                let auth = match load_auth() {
                    Ok(a) => a,
                    Err(e) => { eprintln!("{}", e); std::process::exit(1); }
                };
                let endpoint = format!("{}/api/model/validations", base);
                match client
                    .get(endpoint)
                    .header(AUTHORIZATION, format!("Bearer {}", auth.access_token))
                    .send()
                {
                    Ok(resp) => {
                        let status = resp.status();
                        match resp.text() {
                            Ok(body) => {
                                if status.is_success() {
                                    pretty_print_pending_validations(&body);
                                    std::process::exit(0);
                                } else {
                                    eprintln!("List failed ({}): {}", status, body);
                                    std::process::exit(1);
                                }
                            }
                            Err(e) => { eprintln!("Failed to read response body: {}", e); std::process::exit(1); }
                        }
                    }
                    Err(e) => { eprintln!("HTTP request error: {}", e); std::process::exit(1); }
                }
            }

            match sub_m.subcommand() {
                Some(("list", _)) => {
                    let auth = match load_auth() {
                        Ok(a) => a,
                        Err(e) => { eprintln!("{}", e); std::process::exit(1); }
                    };
                    let endpoint = format!("{}/api/model", base);
                    match client
                        .get(endpoint)
                        .header(AUTHORIZATION, format!("Bearer {}", auth.access_token))
                        .send()
                    {
                        Ok(resp) => {
                            let status = resp.status();
                            match resp.text() {
                                Ok(body) => {
                                    if status.is_success() {
                                        if body.trim().is_empty() || body.trim() == "[]" {
                                            println!("No models found.");
                                        } else {
                                            pretty_print_models(&body);
                                        }
                                        std::process::exit(0);
                                    } else {
                                        eprintln!("List failed ({}): {}", status, body);
                                        std::process::exit(1);
                                    }
                                }
                                Err(e) => { eprintln!("Failed to read response body: {}", e); std::process::exit(1); }
                            }
                        }
                        Err(e) => { eprintln!("HTTP request error: {}", e); std::process::exit(1); }
                    }
                }
                Some(("new", sub_new)) => {
                    #[derive(Serialize)]
                    struct ModelCreate<'a> { vectorFormat: &'a str, name: &'a str, description: Option<&'a str> }

                    let auth = match load_auth() {
                        Ok(a) => a,
                        Err(e) => { eprintln!("{}", e); std::process::exit(1); }
                    };

                    let vector_format = sub_new.get_one::<String>("vector-format").map(String::as_str).expect("--vector-format is required");
                    let name = sub_new.get_one::<String>("name").map(String::as_str).expect("--name is required");
                    let description = sub_new.get_one::<String>("description").map(String::as_str);

                    let payload = ModelCreate { vectorFormat: vector_format, name, description };
                    let endpoint = format!("{}/api/model", base);
                    println!("Creating model '{}'...", name);
                    match client
                        .post(endpoint)
                        .header(AUTHORIZATION, format!("Bearer {}", auth.access_token))
                        .header(CONTENT_TYPE, "application/json")
                        .json(&payload)
                        .send()
                    {
                        Ok(resp) => {
                            let status = resp.status();
                            match resp.text() {
                                Ok(body) => {
                                    if status.is_success() { println!("Success: {}", body); std::process::exit(0); }
                                    else { eprintln!("Create failed ({}): {}", status, body); std::process::exit(1); }
                                }
                                Err(e) => { eprintln!("Failed to read response body: {}", e); std::process::exit(1); }
                            }
                        }
                        Err(e) => { eprintln!("HTTP request error: {}", e); std::process::exit(1); }
                    }
                }
                _ => {
                    eprintln!("Use: verse model list | verse model new --vector-format <FORMAT> --name <NAME> [--description <TEXT>]");
                    std::process::exit(2);
                }
            }
        }
        _ => {
            println!("This is V.E.R.S.E, a command line tool to provide a model validation interface that protects the privacy of both the parties involved. Use --help for more information.");
        }
    }
}