from fastapi import FastAPI, status
from fastapi.middleware.cors import CORSMiddleware
from fastapi.responses import JSONResponse
from routes import index, user, model
from middleware.logger import log_requests
from config.db import init_db, close_db
from config.settings import settings
from contextlib import asynccontextmanager


@asynccontextmanager
async def lifespan(app: FastAPI):
    await init_db()
    yield
    await close_db()


app = FastAPI(
    title=settings.api_title,
    version=settings.api_version,
    docs_url="/api/docs",
    redoc_url="/api/redoc",
    openapi_url="/api/openapi.json",
    lifespan=lifespan
)

app.add_middleware(
    CORSMiddleware,
    allow_origins=["*"],
    allow_credentials=True,
    allow_methods=["*"],
    allow_headers=["*"],
)

# log - middleware
app.middleware("http")(log_requests)

# Routers
app.include_router(index.router, prefix="/api", tags=["General"])
app.include_router(user.router, prefix="/api/users", tags=["Users"])
app.include_router(model.router, prefix="/api/model", tags=["Models"])

# Root route
@app.get("/")
def read_main_root():
    return JSONResponse(
        status_code=status.HTTP_200_OK,
        content={"message": "Welcome welcome!! VERSE API"}
    )

# Catch-all route
@app.get("/{full_path:path}")
def catch_all(full_path: str):
    return JSONResponse(
        status_code=status.HTTP_404_NOT_FOUND,
        content={"message": "Route not found", "path": full_path}
    )