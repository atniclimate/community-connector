# Blueprint: app state machine + wasm worker integration (Phase 3, director)

Sources: AGENTS.md I4 (app state mutates ONLY through app/src/state - the
predecessor mutated viewState in 20+ places and paid for it), ADR-003 D3/D5
and amendments (worker owns the CnApi instance; monotonic revision
enforcement is a stated obligation of this state machine). TypeScript
strict; no frameworks; no new runtime dependencies.

## app/src/state/ - the single mutation point

- `state.ts`: `AppState` - one readonly object tree:
  ```ts
  interface AppState {
    readonly session: {
      readonly groupId: string | null;
      readonly viewer: ViewerContextDto;        // mirrors cn-perm ViewerContext JSON
      readonly revision: number;                // last ACCEPTED projection revision
      readonly loadState: "idle" | "loading" | "ready" | "error";
      readonly lastError: ErrorEnvelopeDto | null;
    };
    readonly view: {
      readonly mode: "overview" | "focus" | "story";
      readonly focusedEntityId: string | null;  // focus mode only
      readonly storyId: string | null;          // story mode only
      readonly storyStep: number;               // story mode only
    };
    readonly ui: {
      readonly legendOpen: boolean;
      readonly sidebarOpen: boolean;
      readonly detailEntityId: string | null;
      readonly reducedMotion: boolean;          // from prefers-reduced-motion
    };
    readonly data: {
      readonly projection: ProjectionDto | null;
      readonly detail: EntityDetailDto | null;
    };
  }
  ```
- `actions.ts`: a closed discriminated union `Action` (kind field). Include
  at minimum: group load lifecycle (requested/succeeded/failed), projection
  received {projection, revision}, entity focused/cleared, detail
  received/cleared, story entered/stepped/exited, legend/sidebar toggled,
  reduced-motion changed, error surfaced/dismissed.
- `reducer.ts`: `reduce(state, action): AppState` - PURE (no IO, no Date,
  no random). CRITICAL RULE (ADR-003 amendment): a `projection received`
  action with `revision <= state.session.revision` returns state UNCHANGED
  (stale worker response dropped) - same for stale detail responses (tag
  detail responses with the revision they were computed at).
- `store.ts`: `createStore(initial)` returning `{ getState, dispatch,
  subscribe }`. `dispatch` is the ONLY mutation path in the entire app.
  Freeze state objects in dev builds (`Object.freeze` deep) so accidental
  mutation throws. No other module may hold mutable state about the domain
  (renderer-internal GPU buffers are display cache, not state).
- `selectors.ts`: pure selectors the viz/ui layers use (focused entity,
  visible story step, projected entity by id, ...).

## app/src/wasm/ - worker boundary (ADR-003 D5)

- `worker.ts` (built as a module worker): owns the single `CnApi` instance
  from the wasm pkg. Handles a typed request protocol.
- `protocol.ts`: request/response types with `correlationId: number`,
  request kinds mirroring the cn-api surface actually used in Phase 3:
  loadGroup (begin/chunks/commit collapsed into one worker-side sequence),
  projection, entityDetail, submitOps, search, queryPaths,
  queryNeighborhood. Response carries the raw envelope string PARSED once
  in the worker; errors cross as the typed envelope, never thrown strings.
- `client.ts`: `WasmClient` - promise-per-correlationId wrapper. It does
  NOT touch the store; callers get promises.
- `effects.ts` (in app/src/state/): the ONLY place promises meet dispatch:
  small named effect functions (loadGroup(store, client, ...),
  refreshProjection(store, client), openDetail(store, client, id)) that
  dispatch request-lifecycle actions and dispatch result actions on
  resolve. Effects read `getState().session.revision` at dispatch time; the
  reducer's stale rule is the enforcement backstop.

## Wire-up

- `app/src/main.ts`: create store; detect `prefers-reduced-motion` and
  dispatch initial ui state + listen for changes; instantiate WasmClient
  with the worker; expose NOTHING mutable on window (dev builds may expose
  a readonly `__cn_state_snapshot()` helper).
- Keep the existing placeholder render working: subscribe to the store and
  render a minimal status line (group load state + projected entity count)
  into #app. The real viz layer arrives in later checklist items and will
  subscribe the same way.

## Test obligations (vitest - add as devDependency; wire "test" npm script)

1. Reducer purity: same (state, action) twice gives identical results;
   unknown action kinds are a type error (compile-time exhaustiveness via
   `never` check).
2. Stale revision: projection revision N applied, then N-1 arrives ->
   state unchanged (same object identity); N+1 applies.
3. Mode transitions: overview -> focus -> story -> exit sequences leave
   consistent view state (no focusedEntityId lingering in story mode, etc.).
4. Store: dispatch notifies subscribers exactly once per action; getState
   identity stable when reducer returns unchanged state.
5. Dev freeze: mutating a dispatched state snapshot throws in dev mode.
6. Protocol: correlation ids route responses to the right promise; an
   error envelope rejects with the typed error; out-of-order responses
   resolve correctly.
   (Worker tests run the protocol against a mock transport - no wasm in
   unit tests; the existing smoke:node covers the real wasm path.)

## Definition of done

From app/: npm run typecheck; npm run build; npm run build:smoke; npm test
(new vitest suite green); root: pwsh scripts/pii-scan.ps1. NO changes under
core/. Record vitest's pinned version in docs/ENVIRONMENT.md's table.
