import {
  AmbientLight,
  BackSide,
  BufferAttribute,
  BufferGeometry,
  Color,
  DirectionalLight,
  FogExp2,
  Mesh,
  Points,
  PointsMaterial,
  ShaderMaterial,
  Scene,
  SphereGeometry,
} from "three";
import type { Theme } from "../theme/tokens";
import { RENDER_COLORS, RENDER_TOKENS } from "./config";

export type SceneSetup = {
  readonly scene: Scene;
  readonly dispose: () => void;
};

const XYZ_COMPONENTS = 3;
const STAR_POINT_SIZE = 1.4;
const STAR_SEED = 42;
const LCG_A = 1664525;
const LCG_C = 1013904223;
const UINT32_MOD = 0x100000000;
const UNIT = 1;
const DOUBLE = 2;
const BACKGROUND_WIDTH_SEGMENTS = 32;
const BACKGROUND_HEIGHT_SEGMENTS = 16;
const BACKGROUND_RADIUS_MULTIPLIER = 2;
const VIGNETTE_EDGE = 0.85;
const DITHER_SCALE = 255;

function token(theme: Theme | null, key: string, fallback: string): string {
  return theme?.tokens[key]?.hex ?? fallback;
}

function lcg(seed: number): () => number {
  let state = seed >>> 0;
  return () => {
    state = (Math.imul(state, LCG_A) + LCG_C) >>> 0;
    return state / UINT32_MOD;
  };
}

function starfield(): Points {
  const random = lcg(STAR_SEED);
  const positions: number[] = [];
  for (let index = 0; index < RENDER_TOKENS.scene.stars; index += UNIT) {
    positions.push(
      (random() * DOUBLE - UNIT) * RENDER_TOKENS.scene.starRadius,
      (random() * DOUBLE - UNIT) * RENDER_TOKENS.scene.starRadius,
      (random() * DOUBLE - UNIT) * RENDER_TOKENS.scene.starRadius,
    );
  }
  const geometry = new BufferGeometry();
  geometry.setAttribute("position", new BufferAttribute(new Float32Array(positions), XYZ_COMPONENTS));
  const material = new PointsMaterial({
    size: STAR_POINT_SIZE,
    color: RENDER_COLORS.keyLight,
    transparent: true,
    opacity: RENDER_TOKENS.scene.starMinBrightness,
    depthWrite: false,
  });
  return new Points(geometry, material);
}

function background(center: string, edge: string): Mesh<SphereGeometry, ShaderMaterial> {
  const geometry = new SphereGeometry(
    RENDER_TOKENS.scene.starRadius * BACKGROUND_RADIUS_MULTIPLIER,
    BACKGROUND_WIDTH_SEGMENTS,
    BACKGROUND_HEIGHT_SEGMENTS,
  );
  const material = new ShaderMaterial({
    side: BackSide,
    depthWrite: false,
    uniforms: {
      uCenter: { value: new Color(center) },
      uEdge: { value: new Color(edge) },
      uVignetteEdge: { value: VIGNETTE_EDGE },
      uDitherScale: { value: DITHER_SCALE },
    },
    vertexShader: `
      varying vec3 vWorld;
      void main() {
        vec4 world = modelMatrix * vec4(position, 1.0);
        vWorld = normalize(world.xyz);
        gl_Position = projectionMatrix * viewMatrix * world;
      }
    `,
    fragmentShader: `
      varying vec3 vWorld;
      uniform vec3 uCenter;
      uniform vec3 uEdge;
      uniform float uVignetteEdge;
      uniform float uDitherScale;
      float ign(vec2 p) {
        return fract(52.9829189 * fract(dot(p, vec2(0.06711056, 0.00583715))));
      }
      void main() {
        float radial = smoothstep(0.0, uVignetteEdge, length(vWorld.xy));
        vec3 color = mix(uCenter, uEdge, radial);
        color += (ign(gl_FragCoord.xy) - 0.5) / uDitherScale;
        gl_FragColor = vec4(color, 1.0);
      }
    `,
  });
  return new Mesh(geometry, material);
}

export function createVizScene(theme: Theme | null): SceneSetup {
  const scene = new Scene();
  const bgEdge = token(theme, "bg.edge", RENDER_COLORS.fallbackBackgroundEdge);
  const bgCenter = token(theme, "bg.center", RENDER_COLORS.fallbackBackgroundCenter);
  const bg = background(bgCenter, bgEdge);
  scene.background = new Color(bgEdge);
  scene.fog = new FogExp2(bgEdge, RENDER_TOKENS.scene.fogDensity);
  const key = new DirectionalLight(RENDER_COLORS.keyLight, RENDER_TOKENS.scene.keyIntensity);
  key.position.set(UNIT, DOUBLE, UNIT);
  const fill = new DirectionalLight(RENDER_COLORS.fillLight, RENDER_TOKENS.scene.fillIntensity);
  fill.position.set(-UNIT, -UNIT, UNIT);
  const ambient = new AmbientLight(RENDER_COLORS.keyLight, RENDER_TOKENS.scene.ambientIntensity);
  const stars = starfield();
  scene.add(bg, key, fill, ambient, stars);
  return {
    scene,
    dispose: () => {
      bg.geometry.dispose();
      bg.material.dispose();
      stars.geometry.dispose();
      (stars.material as PointsMaterial).dispose();
    },
  };
}
