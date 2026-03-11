const API_BASE_URL = process.env.NEXT_PUBLIC_API_URL ?? "http://localhost:8080";

interface ApiError {
  code: string;
  message: string;
  details?: unknown;
}

interface ApiResponse<T> {
  data?: T;
  error?: ApiError;
}

class ApiClient {
  private baseUrl: string;

  constructor(baseUrl: string) {
    this.baseUrl = baseUrl;
  }

  private async request<T>(
    method: string,
    path: string,
    options?: {
      body?: unknown;
      params?: Record<string, string>;
    }
  ): Promise<ApiResponse<T>> {
    const url = new URL(path, this.baseUrl);

    if (options?.params) {
      Object.entries(options.params).forEach(([key, value]) => {
        url.searchParams.append(key, value);
      });
    }

    const headers: HeadersInit = {
      "Content-Type": "application/json",
    };

    try {
      const response = await fetch(url.toString(), {
        method,
        headers,
        body: options?.body ? JSON.stringify(options.body) : undefined,
      });

      if (!response.ok) {
        const error = await response.json().catch(() => ({
          code: "SAHI_5000",
          message: "An unexpected error occurred",
        }));

        return { error };
      }

      const data = await response.json();
      return { data };
    } catch {
      return {
        error: {
          code: "SAHI_5001",
          message: "Network error. Please check your connection.",
        },
      };
    }
  }

  get<T>(path: string, params?: Record<string, string>) {
    return this.request<T>("GET", path, { params });
  }

  post<T>(path: string, body: unknown) {
    return this.request<T>("POST", path, { body });
  }
}

export const apiClient = new ApiClient(API_BASE_URL);
