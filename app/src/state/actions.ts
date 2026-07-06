import type {
  EntityDetailDto,
  ErrorEnvelopeDto,
  ProjectionDto,
  ViewerContextDto,
} from "./state";
import type { Theme, ThemeReport } from "../theme/tokens";

export type Action =
  | {
      readonly kind: "groupLoadRequested";
      readonly groupId: string;
      readonly viewer: ViewerContextDto;
    }
  | {
      readonly kind: "groupLoadSucceeded";
      readonly groupId: string;
    }
  | {
      readonly kind: "groupLoadFailed";
      readonly groupId: string;
      readonly error: ErrorEnvelopeDto;
    }
  | {
      readonly kind: "projectionReceived";
      readonly projection: ProjectionDto;
      readonly revision: number;
    }
  | {
      readonly kind: "themeDerived";
      readonly theme: Theme;
      readonly report: ThemeReport;
    }
  | {
      readonly kind: "entityFocused";
      readonly entityId: string;
    }
  | {
      readonly kind: "entityFocusCleared";
    }
  | {
      readonly kind: "detailReceived";
      readonly detail: EntityDetailDto;
      readonly entityId: string;
      readonly revision: number;
    }
  | {
      readonly kind: "detailCleared";
    }
  | {
      readonly kind: "storyEntered";
      readonly storyId: string;
      readonly step: number;
    }
  | {
      readonly kind: "storyStepped";
      readonly step: number;
    }
  | {
      readonly kind: "storyExited";
    }
  | {
      readonly kind: "legendToggled";
    }
  | {
      readonly kind: "sidebarToggled";
    }
  | {
      readonly kind: "reducedMotionChanged";
      readonly reducedMotion: boolean;
    }
  | {
      readonly kind: "errorSurfaced";
      readonly error: ErrorEnvelopeDto;
    }
  | {
      readonly kind: "errorDismissed";
    };
