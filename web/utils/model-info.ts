/**
 * Model Information Utility
 * 
 * Fetches and caches model information from models.dev API
 */

export interface ModelInfo {
  name: string;
  limit: {
    context: number;
    input: number;
    output: number;
  };
  cost: {
    input: number;
    output: number;
    reasoning?: number;
    cache_read?: number;
    cache_write?: number;
  };
  modalities: {
    input: string[];
    output: string[];
  };
  tool_call: boolean;
  reasoning: boolean;
}

interface ModelsDevResponse {
  [providerId: string]: {
    models: {
      [modelId: string]: ModelInfo;
    };
  };
}

class ModelInfoCache {
  private cache: Map<string, ModelInfo> = new Map();
  private lastFetch: number = 0;
  private readonly CACHE_DURATION = 24 * 60 * 60 * 1000; // 24 hours
  private readonly API_URL = '/api/models'; // Proxy through our server to avoid CORS
  private fetchPromise: Promise<void> | null = null;

  async init(): Promise<void> {
    const now = Date.now();
    if (this.cache.size > 0 && now - this.lastFetch < this.CACHE_DURATION) {
      return;
    }

    // Prevent multiple simultaneous fetches
    if (this.fetchPromise) {
      return this.fetchPromise;
    }

    this.fetchPromise = this.fetchModels();
    try {
      await this.fetchPromise;
    } finally {
      this.fetchPromise = null;
    }
  }

  private async fetchModels(): Promise<void> {
    try {
      console.log('[model-info] Fetching model data from models.dev...');
      const response = await fetch(this.API_URL);
      
      if (!response.ok) {
        throw new Error(`Failed to fetch models: ${response.statusText}`);
      }

      const data: ModelsDevResponse = await response.json();
      
      // Parse and cache all models
      let modelCount = 0;
      for (const [providerId, provider] of Object.entries(data)) {
        if (provider.models) {
          for (const [modelId, modelInfo] of Object.entries(provider.models)) {
            // Store with full model ID (provider/model)
            const fullModelId = modelId.includes('/') ? modelId : `${providerId}/${modelId}`;
            this.cache.set(fullModelId, modelInfo);
            
            // Also store without provider prefix for convenience
            const shortModelId = modelId.split('/').pop() || modelId;
            if (!this.cache.has(shortModelId)) {
              this.cache.set(shortModelId, modelInfo);
            }
            
            modelCount++;
          }
        }
      }

      this.lastFetch = Date.now();
      console.log(`[model-info] Cached ${modelCount} models from ${Object.keys(data).length} providers`);
    } catch (error) {
      console.error('[model-info] Failed to fetch model data:', error);
      // Don't throw - allow app to continue with estimation
    }
  }

  getModelInfo(modelId: string): ModelInfo | null {
    // Try exact match first
    let info = this.cache.get(modelId);
    if (info) return info;

    // Try without provider prefix
    const shortId = modelId.split('/').pop();
    if (shortId) {
      info = this.cache.get(shortId);
      if (info) return info;
    }

    // Try case-insensitive match
    const lowerModelId = modelId.toLowerCase();
    for (const [key, value] of this.cache.entries()) {
      if (key.toLowerCase() === lowerModelId) {
        return value;
      }
    }

    return null;
  }

  getDefaultLimits(): { context: number; input: number; output: number } {
    // Default to GPT-4 limits if model not found
    return {
      context: 128000,
      input: 128000,
      output: 4096
    };
  }
}

export const modelInfoCache = new ModelInfoCache();

/**
 * Get model information by model ID
 */
export function getModelInfo(modelId: string): ModelInfo | null {
  return modelInfoCache.getModelInfo(modelId);
}

/**
 * Get context window limit for a model
 */
export function getContextLimit(modelId: string): number {
  const info = getModelInfo(modelId);
  return info?.limit?.context || modelInfoCache.getDefaultLimits().context;
}

/**
 * Get input token limit for a model
 */
export function getInputLimit(modelId: string): number {
  const info = getModelInfo(modelId);
  return info?.limit?.input || modelInfoCache.getDefaultLimits().input;
}

/**
 * Get output token limit for a model
 */
export function getOutputLimit(modelId: string): number {
  const info = getModelInfo(modelId);
  return info?.limit?.output || modelInfoCache.getDefaultLimits().output;
}

/**
 * Calculate estimated cost for token usage
 */
export function estimateCost(modelId: string, inputTokens: number, outputTokens: number): number {
  const info = getModelInfo(modelId);
  if (!info?.cost) return 0;

  const inputCost = (inputTokens / 1_000_000) * info.cost.input;
  const outputCost = (outputTokens / 1_000_000) * info.cost.output;
  
  return inputCost + outputCost;
}
