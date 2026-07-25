import type {
  EntityDetailDto,
  ErrorEnvelopeDto,
  IntakeRecordSummaryDto,
  KindMeta,
  ProjectionDto,
  QualityTierName,
  RequestIdentity,
  SearchHitDto,
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
      readonly kindMeta: Readonly<Record<string, KindMeta>>;
    }
  | {
      readonly kind: "entityFocused";
      readonly entityId: string;
    }
  | {
      readonly kind: "entityHovered";
      readonly entityId: string | null;
    }
  | {
      readonly kind: "entityFocusCleared";
    }
  | {
      readonly kind: "detailRequested";
      readonly entityId: string;
      readonly request: RequestIdentity;
    }
  | {
      readonly kind: "detailReceived";
      readonly detail: EntityDetailDto;
      readonly entityId: string;
      readonly request: RequestIdentity;
    }
  | {
      readonly kind: "detailFailed";
      readonly entityId: string;
      readonly request: RequestIdentity;
      readonly error: ErrorEnvelopeDto;
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
      readonly kind: "qualityTierChanged";
      readonly tier: QualityTierName;
    }
  | {
      readonly kind: "searchRequested";
      readonly query: string;
      readonly request: RequestIdentity;
    }
  | {
      readonly kind: "searchSucceeded";
      readonly query: string;
      readonly hits: readonly SearchHitDto[];
      readonly request: RequestIdentity;
    }
  | {
      readonly kind: "searchFailed";
      readonly query: string;
      readonly error: ErrorEnvelopeDto;
      readonly request: RequestIdentity;
    }
  | {
      readonly kind: "searchCleared";
    }
  | {
      readonly kind: "errorSurfaced";
      readonly error: ErrorEnvelopeDto;
    }
  | {
      readonly kind: "errorDismissed";
    }
  | {
      readonly kind: "intakeDirGranted";
      readonly dirName: string;
    }
  | {
      readonly kind: "intakeDirCleared";
    }
  | {
      readonly kind: "intakeScanStarted";
    }
  | {
      readonly kind: "intakeQueueLoaded";
      readonly records: readonly IntakeRecordSummaryDto[];
      readonly pendingDecisionFiles: number;
    }
  | {
      readonly kind: "intakeRecordStaged";
      readonly record: IntakeRecordSummaryDto;
    }
  | {
      readonly kind: "intakeReviewDecided";
      readonly recordId: string;
    }
  | {
      readonly kind: "intakeFailed";
      readonly error: ErrorEnvelopeDto;
    };
