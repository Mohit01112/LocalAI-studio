export interface LocalModel {
  id: string;
  name: string;
  size?: number;
  format?: string;
}

export interface RuntimeInfo {
  id: string;
  name: string;
  version?: string;
  installed: boolean;
  running: boolean;
}

export interface RuntimeProvider {
  readonly id: string;
  readonly name: string;

  detect(): Promise<RuntimeInfo>;

  getModels(): Promise<LocalModel[]>;

  downloadModel(modelId: string): Promise<void>;

  deleteModel(modelId: string): Promise<void>;

  start(): Promise<void>;

  stop(): Promise<void>;

  chat(
    modelId: string,
    message: string
  ): Promise<string>;
}