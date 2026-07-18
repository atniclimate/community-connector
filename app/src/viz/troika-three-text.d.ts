// Minimal ambient typing for troika-three-text 0.52.x (ships no TypeScript types).
// Only the surface used by app/src/viz/labels.ts is declared.
declare module "troika-three-text" {
  import type { Color, Material, Mesh } from "three";

  export class Text extends Mesh {
    public text: string;
    public font: string | null;
    public fontSize: number;
    public color: string | number | Color;
    public outlineColor: string | number | Color;
    public outlineWidth: number | string;
    public anchorX: number | "left" | "center" | "right";
    public anchorY:
      | number
      | "top"
      | "top-baseline"
      | "middle"
      | "bottom-baseline"
      | "bottom";
    public maxWidth: number;
    public depthOffset: number;
    public override material: Material;
    public sync(callback?: () => void): void;
    public dispose(): void;
  }
}
