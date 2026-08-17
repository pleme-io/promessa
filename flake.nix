{
  description = "Typed business-outcome substrate (Portuguese — promise) — the Viggy Method's core types and runtime. Declares PromessaCR, TargetController trait, TypedAction enum, RemediationPolicy, Severity, PromessaTargetKind, AuthoringDecision, AnomalyEmission, the ten typed legs (Declaration/Admission/Registration/Tick/Observation/Diff/Action/Attestation/Anomaly/Audit) per pleme-io/theory/VIGGY-LEGOS.md, the Seven-Beat Convergence Tick per pleme-io/theory/CONTINUOUS-SOLUTION-MACHINE.md, the OutcomeChain leaf pipeline, and the gRPC/REST/GraphQL/MCP API surfaces auto-derived from spec/promessa.{proto,openapi.yaml,graphql}. First TargetController kind (Security) lives in pleme-io/engenho-promessa-controllers; the first real promessa is a FedRAMP SCR declaration authored in a downstream consumer repo. Spec: theory/VIGGY-LEGOS.md, theory/VIGGY-AUTHORING.md, theory/CONTINUOUS-SOLUTION-MACHINE.md.";

  # substrate.rust.library dispatches over Cargo.gen.lock (the slim gen delta,
  # reconstructed to the full BuildSpec in pure Nix) — no crate2nix, no Cargo.nix.
  inputs.substrate.url = "github:pleme-io/substrate";

  outputs = { substrate, ... }: substrate.rust.library {
    src = ./.;
    member = "promessa";
  };
}
