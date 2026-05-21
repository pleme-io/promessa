{
  description = "Typed business-outcome substrate (Portuguese — promise) — the Viggy Method's core types and runtime. Declares PromessaCR, TargetController trait, TypedAction enum, RemediationPolicy, Severity, PromessaTargetKind, AuthoringDecision, AnomalyEmission, the ten typed legs (Declaration/Admission/Registration/Tick/Observation/Diff/Action/Attestation/Anomaly/Audit) per pleme-io/theory/VIGGY-LEGOS.md, the Seven-Beat Convergence Tick per pleme-io/theory/CONTINUOUS-SOLUTION-MACHINE.md, the OutcomeChain leaf pipeline, and the gRPC/REST/GraphQL/MCP API surfaces auto-derived from spec/promessa.{proto,openapi.yaml,graphql}. First TargetController kind (Security) lives in pleme-io/engenho-promessa-controllers; first real promessa lives at akeylesslabs/akeyless-nix-images/promessas/akeyless-nix-images-fedramp-scr.tatara (ASM-17571). Spec: theory/VIGGY-LEGOS.md, theory/VIGGY-AUTHORING.md, theory/CONTINUOUS-SOLUTION-MACHINE.md.";
  inputs = {
    nixpkgs = {
      url = "github:nixos/nixpkgs?ref=nixos-unstable";
    };
    flake-utils = {
      url = "github:numtide/flake-utils";
    };
    substrate = {
      url = "github:pleme-io/substrate";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    tatara = {
      url = "git+ssh://git@github.com/pleme-io/tatara";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    shikumi = {
      url = "git+ssh://git@github.com/pleme-io/shikumi";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    shigoto = {
      url = "git+ssh://git@github.com/pleme-io/shigoto";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    cofre = {
      url = "git+ssh://git@github.com/pleme-io/cofre";
      inputs.nixpkgs.follows = "nixpkgs";
    };
    tameshi = {
      url = "git+ssh://git@github.com/pleme-io/tameshi";
      inputs.nixpkgs.follows = "nixpkgs";
    };
  };
  outputs = inputs @ { self, nixpkgs, crate2nix, fenix, substrate, ... }:
    (import "${substrate}/lib/rust-library-workspace-flake.nix" {
      inherit nixpkgs crate2nix fenix;
    }) {
      workspaceName = "promessa";
      members = [ "promessa-types" "promessa-store" "promessa-eval" "promessa-runtime" "promessa-rpc" "promessa-graphql" "promessa-rest" "promessa-mcp" "promessa-cli" "promessa" ];
      src = self;
    };
}
