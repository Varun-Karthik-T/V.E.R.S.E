import "./App.css";
import { BrowserRouter, Routes, Route } from "react-router-dom";
import Home from "./pages/Home.tsx";
import Models from "./pages/Models.tsx";
import Docs from "./pages/Docs.tsx";
import Navbar from "./components/Navbar/Bar.tsx";

function App() {
  return (
    <>
      <BrowserRouter>
        <Navbar />
        <div className="min-h-screen pt-28 px-4 sm:px-6 lg:px-8">
          <div className="mx-auto max-w-6xl">
            <Routes>
              <Route path="/" element={<Home />} />
              <Route path="/models" element={<Models />} />
              <Route path="/docs" element={<Docs />} />
            </Routes>
          </div>
        </div>
      </BrowserRouter>
    </>
  );
}

export default App;
