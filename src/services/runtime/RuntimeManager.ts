import type {
  RuntimeInfo,
  RuntimeProvider,
} from "./RuntimeProvider";

export class RuntimeManager {
  private providers: RuntimeProvider[] = [];

  registerProvider(provider: RuntimeProvider) {
    this.providers.push(provider);
  }

  async discoverRuntimes(): Promise<RuntimeInfo[]> {
    const results: RuntimeInfo[] = [];

    for (const provider of this.providers) {
      try {
        const runtime = await provider.detect();

        if (runtime.installed) {
          results.push(runtime);
        }
      } catch (error) {
        console.error(
          `Failed to detect ${provider.name}:`,
          error
        );
      }
    }

    return results;
  }

  getProviders(): RuntimeProvider[] {
    return this.providers;
  }
}