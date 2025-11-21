import axios from "axios";

const apiClient = axios.create({
    baseURL:  "http://127.0.0.1:8000/api",
});

export default apiClient;
export type Model = {
    id: string;
    name: string;
    userId: string;
    description: string;
    vectorFormat: string;
    createdAt: string;
    updatedAt: string;
};

export const fetchModels = async (): Promise<Model[]> => {
    const response = await apiClient.get<Model[]>("/model/");
    console.log("here's the response.data", response.data);
    return response.data;
}