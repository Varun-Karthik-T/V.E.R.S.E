from fastapi import APIRouter, HTTPException, status, Depends, UploadFile, File, Form
from controller.model import (
    create_model, get_user_models,  get_model_validation_requests, get_user_models_with_validations,
    create_validation_request_with_file, get_all_models_controller, add_proof_to_validation, get_verifier_validation_requests_controller, get_particular_validation_request
)
from schemas.model import (
    ModelCreate, ModelResponse, 
    ValidationRequestResponse, UserModelsWithValidationsResponse,

)
from utils.auth import get_current_user
from models.user import User
from typing import List, Optional

router = APIRouter()

@router.post("", response_model=ModelResponse, status_code=status.HTTP_201_CREATED)
async def create_new_model(
    model_data: ModelCreate,
    current_user: User = Depends(get_current_user)
):
    """Create a new model for the authenticated user"""
    return await create_model(model_data, current_user)

@router.get("/",response_model=List[ModelResponse])
async def get_all_models():
    """Get all models for the authenticated user"""
    return await get_all_models_controller()



@router.get("", response_model=List[ModelResponse])
async def get_models(
    current_user: User = Depends(get_current_user)
):
    """Get all models for the authenticated user"""
    return await get_user_models(current_user)


@router.post("/validation-request", response_model=ValidationRequestResponse, status_code=status.HTTP_201_CREATED)
async def create_new_validation_request(
    model_id: str = Form(...),
    elf_file: UploadFile = File(...),
    hashValue : str = Form(...),
    current_user: User = Depends(get_current_user)
):
    """Create a new validation request for a model with ELF file upload"""
    print(
        f"Received validation request for model_id: {model_id}, "
        f"user: {current_user.email}"
    )
    return await create_validation_request_with_file(model_id, elf_file,hashValue, current_user)

@router.get("/{model_id}/validation-requests", response_model=List[ValidationRequestResponse])
async def get_validation_requests(
    model_id: str,
    current_user: User = Depends(get_current_user)
):
    """Get all validation requests for a specific model"""
    return await get_model_validation_requests(model_id, current_user)

@router.get("/validations", response_model=UserModelsWithValidationsResponse)
async def get_models_with_validations(
    current_user: User = Depends(get_current_user)
):
    """Get all user models with their validation requests"""
    return await get_user_models_with_validations(current_user)

@router.get("/validation-requests/verifier", response_model=List[ValidationRequestResponse])
async def get_verifier_validation_requests(
    current_user: User = Depends(get_current_user)
):
    """Get all validation requests placed by a verifier"""
    return await get_verifier_validation_requests_controller(current_user)

@router.put("/proof/{validation_request_id}",response_model = ValidationRequestResponse) 
async def add_proof_to_validation_request(
    validation_request_id: str,
    json_file: UploadFile = File(...),
    current_user: User = Depends(get_current_user)
):
    """Append JSON proof file to an existing validation request"""
    return await add_proof_to_validation(validation_request_id, json_file, current_user)

@router.get("/validation-request/{validation_request_id}",response_model = ValidationRequestResponse) 
async def get_validation_request_by_id(
    validation_request_id: str,
    current_user: User = Depends(get_current_user)
):
    """Get a specific validation request by its ID"""
    validation_request = await get_particular_validation_request(validation_request_id)
    if validation_request:  
        return validation_request
    raise HTTPException(status_code=status.HTTP_404_NOT_FOUND, detail="Validation request not found")