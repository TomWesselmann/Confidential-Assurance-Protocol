/**
 * CAP REST API Client
 * Source: WEBUI_BACKEND_STATUS.md Section 1
 *
 * @description Type-safe HTTP client for CAP backend
 * @security OAuth2 Bearer Token authentication
 *
 * Architecture:
 * - Functional Core: Pure type transformations
 * - Imperative Shell: HTTP I/O via axios
 */

import axios, { type AxiosInstance, AxiosError } from 'axios';
import type {
  HealthResponse,
  ReadinessResponse,
  VerifyRequest,
  VerifyResponse,
  UploadResponse,
  ApiError,
} from './types';
import { isApiError } from './types';

export class CAPApiClient {
  private client: AxiosInstance;

  constructor(baseURL: string = 'http://localhost:8080', bearerToken?: string) {
    this.client = axios.create({
      baseURL,
      timeout: 30000, // 30s timeout
      headers: {
        'Content-Type': 'application/json',
        ...(bearerToken && { Authorization: `Bearer ${bearerToken}` }),
      },
    });

    // Response interceptor for error handling
    this.client.interceptors.response.use(
      (response) => response,
      (error: AxiosError) => {
        if (error.response?.data && isApiError(error.response.data)) {
          throw error.response.data;
        }
        throw {
          error: 'NetworkError',
          message: error.message,
          status_code: error.response?.status || 0,
          timestamp: new Date().toISOString(),
        } as ApiError;
      }
    );
  }

  /**
   * Health Check
   * GET /healthz
   * @public No authentication required
   */
  async healthCheck(): Promise<HealthResponse> {
    const response = await this.client.get<HealthResponse>('/healthz');
    return response.data;
  }

  /**
   * Readiness Check
   * GET /readyz
   * @public No authentication required
   */
  async readinessCheck(): Promise<ReadinessResponse> {
    const response = await this.client.get<ReadinessResponse>('/readyz');
    return response.data;
  }

  /**
   * Verify Proof Bundle
   * POST /verify
   * @protected Requires OAuth2 Bearer Token (or mock mode in development)
   * @deterministic Same inputs → same verification result
   * @description Backend API v0.11.0 - Requires policy_id + context format
   */
  async verifyProofBundle(request: VerifyRequest): Promise<VerifyResponse> {
    const response = await this.client.post<VerifyResponse>('/verify', request);
    return response.data;
  }

  /**
   * Upload Proof Package (ZIP)
   * POST /proof/upload
   * @protected Requires OAuth2 Bearer Token
   * @description Uploads a ZIP file containing proof package, extracts manifest + proof
   */
  async uploadProofPackage(file: File): Promise<UploadResponse> {
    const formData = new FormData();
    formData.append('file', file);

    const response = await this.client.post<UploadResponse>('/proof/upload', formData, {
      headers: {
        'Content-Type': 'multipart/form-data',
      },
    });
    return response.data;
  }

  /**
   * Update Base URL
   * @description Updates the backend API base URL
   */
  setBaseURL(url: string): void {
    this.client.defaults.baseURL = url;
  }

  /**
   * Update Bearer Token
   * @description Updates the Authorization header for protected endpoints
   */
  setBearerToken(token: string): void {
    this.client.defaults.headers.common['Authorization'] = `Bearer ${token}`;
  }

  /**
   * Remove Bearer Token
   * @description Removes the Authorization header
   */
  clearBearerToken(): void {
    delete this.client.defaults.headers.common['Authorization'];
  }
}

/**
 * Singleton instance for default usage
 * @note Can be reconfigured via setBearerToken()
 */
export const capApiClient = new CAPApiClient();
