import { RENDER_TOKENS } from "./config";

export type QualityTier = "A" | "B" | "C" | "D";

export type QualityProfile = {
  readonly tier: QualityTier;
  readonly dpr: number;
  readonly halos: "full" | "culled" | "off";
};

const ORDER: readonly QualityTier[] = ["A", "B", "C", "D"];
const FIRST_INDEX = 0;
const FRAME_COUNT_START = 0;
const TIER_STEP = 1;
const EMA_REMAINDER = 1;

export function profileForTier(tier: QualityTier): QualityProfile {
  switch (tier) {
    case "A":
      return { tier, dpr: RENDER_TOKENS.dpr.tierA, halos: "full" };
    case "B":
      return { tier, dpr: RENDER_TOKENS.dpr.tierB, halos: "culled" };
    case "C":
      return { tier, dpr: RENDER_TOKENS.dpr.tierC, halos: "off" };
    case "D":
      return { tier, dpr: RENDER_TOKENS.dpr.tierD, halos: "off" };
    default:
      return { tier: "B", dpr: RENDER_TOKENS.dpr.tierB, halos: "culled" };
  }
}

export class QualityManager {
  private tierIndex = ORDER.indexOf("B");
  private emaMs: number = RENDER_TOKENS.quality.upgradeMs;
  private degradeFrames = FRAME_COUNT_START;
  private upgradeFrames = FRAME_COUNT_START;

  public sample(frameMs: number): QualityProfile {
    this.emaMs = this.emaMs * (EMA_REMAINDER - RENDER_TOKENS.quality.emaAlpha) +
      frameMs * RENDER_TOKENS.quality.emaAlpha;
    this.updateCounters();
    this.applyTransition();
    return this.profile;
  }

  public get profile(): QualityProfile {
    return profileForTier(ORDER[this.tierIndex] ?? "D");
  }

  public get ema(): number {
    return this.emaMs;
  }

  private updateCounters(): void {
    this.degradeFrames = this.emaMs > RENDER_TOKENS.quality.degradeMs
      ? this.degradeFrames + TIER_STEP
      : FRAME_COUNT_START;
    this.upgradeFrames = this.emaMs < RENDER_TOKENS.quality.upgradeMs
      ? this.upgradeFrames + TIER_STEP
      : FRAME_COUNT_START;
  }

  private applyTransition(): void {
    if (this.degradeFrames >= RENDER_TOKENS.quality.degradeFrames) {
      this.tierIndex = Math.min(ORDER.length - TIER_STEP, this.tierIndex + TIER_STEP);
      this.degradeFrames = FRAME_COUNT_START;
      this.upgradeFrames = FRAME_COUNT_START;
      return;
    }
    if (this.upgradeFrames >= RENDER_TOKENS.quality.upgradeFrames) {
      this.tierIndex = Math.max(FIRST_INDEX, this.tierIndex - TIER_STEP);
      this.degradeFrames = FRAME_COUNT_START;
      this.upgradeFrames = FRAME_COUNT_START;
    }
  }
}
