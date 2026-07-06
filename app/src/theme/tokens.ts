export type TokenSource = "template" | "default" | "adjusted";

export type ThemeToken = {
  readonly hex: string;
  readonly source: TokenSource;
};

export type Theme = {
  readonly schema_version: "0.1.0";
  readonly tokens: Readonly<Record<string, ThemeToken>>;
};

export type ThemeAdjustment = {
  readonly token: string;
  readonly from: string;
  readonly to: string;
  readonly reason: string;
};

export type ThemeWarning = {
  readonly code: string;
  readonly message: string;
  readonly tokens: readonly string[];
};

export type ThemeReport = {
  readonly schema_version: "0.1.0";
  readonly adjustments: readonly ThemeAdjustment[];
  readonly warnings: readonly ThemeWarning[];
};

export type DerivedTheme = {
  readonly theme: Theme;
  readonly report: ThemeReport;
};

export type GroupTemplateKindDto = {
  readonly id: string;
  readonly color_role: string;
};

export type GroupTemplateDto = {
  readonly schema_version: string;
  readonly kinds: readonly GroupTemplateKindDto[];
  readonly theme?: {
    readonly roles?: Readonly<Record<string, string>>;
  };
};
