//! Focused ForgeCore adapter coverage.
//!
//! The native-backed tests use ForgeCore's deterministic tiny GGUF fixture
//! through FORGE_TEST_MODEL. The binary is deliberately not stored in this
//! workspace; see forgeCORE/docs/NATIVE.md and the Phase-2 handoff notes.

use std::path::{Path, PathBuf};

use forge_core::{Model as NativeModel, TokenAttr, Tokenizer as NativeTokenizer};
use ramforge_core::{parse_gguf_file, tokenizer::TokenType, Tokenizer as RamforgeTokenizer};
use ramforge_runtime::{
    sampling::Sampler, DeviceRegistry, DeviceSelection, ExecutionConfig, ForgeCoreAdapterError,
    ForgeCoreContextOptions, ForgeCoreModel, ForgeCoreModelMetadata,
};

fn fixture() -> Option<PathBuf> {
    match std::env::var("FORGE_TEST_MODEL") {
        Ok(path) => Some(PathBuf::from(path)),
        Err(_) => {
            println!("SKIP: FORGE_TEST_MODEL not set (use ForgeCore's tiny GGUF fixture)");
            None
        }
    }
}

fn load_fixture() -> Option<(ForgeCoreModel, ForgeCoreModelMetadata)> {
    let path = fixture()?;
    let model = ForgeCoreModel::load(&path).expect("deterministic ForgeCore fixture must load");
    let metadata = model.metadata().clone();
    Some((model, metadata))
}

#[test]
fn forgecore_selection_errors_are_propagated_without_fallback() {
    let missing_id = ExecutionConfig::new(
        DeviceSelection::by_id("missing-device"),
        1,
        None,
        false,
    )
    .expect("selection syntax is valid");
    let error = ForgeCoreModel::load_for_execution(
        "/definitely/not/a/model.gguf",
        missing_id,
        &DeviceRegistry::cpu_only(),
    )
    .unwrap_err();
    assert!(matches!(error, ForgeCoreAdapterError::DeviceSelection(_)));

    let no_cpu = ExecutionConfig::new(DeviceSelection::Cpu, 1, None, false)
        .expect("CPU selection is valid");
    let error = ForgeCoreModel::load_for_execution(
        "/definitely/not/a/model.gguf",
        no_cpu,
        &DeviceRegistry::from_devices(Vec::new()),
    )
    .unwrap_err();
    assert!(matches!(error, ForgeCoreAdapterError::DeviceSelection(_)));
}

#[test]
fn forgecore_model_and_context_use_native_metadata() {
    let Some((model, metadata)) = load_fixture() else {
        return;
    };
    assert_eq!(metadata.n_params, 1176);
    assert_eq!(metadata.vocab_size, 32);
    assert_eq!(metadata.size_bytes, 4704);
    assert_eq!(metadata.context_length, 64);
    assert_eq!(metadata.embedding_length, 8);
    assert_eq!(metadata.layer_count, 1);
    assert_eq!(metadata.head_count, 2);
    assert_eq!(metadata.kv_head_count, 2);
    assert!(metadata.description.contains("llama"));

    let session = model
        .open_context(ForgeCoreContextOptions::default())
        .expect("default CPU context must open");
    // ForgeCore exposes the effective native length, including its 256-token
    // padding; model metadata above retains the training length (64).
    assert_eq!(session.context_length(), 256);
    assert_eq!(session.vocab_size(), 32);
}

#[test]
fn forgecore_batch_decode_returns_owned_logits_for_last_position() {
    let Some((model, _)) = load_fixture() else {
        return;
    };
    let mut session = model
        .open_context(ForgeCoreContextOptions::default())
        .expect("context");
    let logits = session
        .decode_tokens(&[1, 5, 6, 7], 0)
        .expect("tiny batch must decode");
    assert_eq!(logits.token_index(), 3);
    assert_eq!(logits.n_vocab(), 32);
    assert!(logits.values().iter().all(|value| value.is_finite()));
    assert!(logits.values().iter().any(|value| *value != 0.0));
}

#[test]
fn forgecore_adapter_translates_model_batch_and_decode_errors() {
    let missing = ForgeCoreModel::load("/definitely/not/a/model.gguf").unwrap_err();
    assert!(matches!(missing, ForgeCoreAdapterError::Model(_)));

    let Some((model, _)) = load_fixture() else {
        return;
    };
    let invalid_context = model
        .open_context(ForgeCoreContextOptions {
            n_ctx: None,
            n_threads: 0,
        })
        .unwrap_err();
    assert!(matches!(
        invalid_context,
        ForgeCoreAdapterError::Configuration(_)
    ));

    let mut session = model
        .open_context(ForgeCoreContextOptions::default())
        .expect("context");

    let empty = session.decode_tokens(&[], 0).unwrap_err();
    assert!(matches!(empty, ForgeCoreAdapterError::Batch(_)));

    let out_of_vocab = session.decode_tokens(&[32], 0).unwrap_err();
    assert!(matches!(out_of_vocab, ForgeCoreAdapterError::Batch(_)));

    session
        .decode_tokens(&[1, 5], 0)
        .expect("valid batch for logits error checks");
    let unflagged = session.logits(0).unwrap_err();
    assert!(matches!(unflagged, ForgeCoreAdapterError::Logits(_)));
    let beyond = session.logits(99).unwrap_err();
    assert!(matches!(beyond, ForgeCoreAdapterError::Logits(_)));

    let mut constrained = model
        .open_context(ForgeCoreContextOptions {
            n_ctx: Some(1),
            n_threads: 1,
        })
        .expect("one-token context");
    constrained
        .decode_token(1, 0)
        .expect("first token in one-token context");
    let native_decode_error = constrained.decode_token(2, 0).unwrap_err();
    assert!(matches!(
        native_decode_error,
        ForgeCoreAdapterError::Decode(_)
    ));
}

const GENERATION_PROMPT: &str = "tok3";
const MAX_GENERATED_TOKENS: usize = 2;

#[derive(Debug)]
struct ForgeCoreGenerationRun {
    prompt_tokens: Vec<u32>,
    generated_token_ids: Vec<u32>,
    logits: Vec<Vec<f32>>,
}

/// Exercise the same public RAMforge adapter calls used by the CLI's
/// ForgeCore branch. The helper returns token IDs and logits so this test can
/// check the generation boundary without depending on rendered text.
fn run_forgecore_generation_once(path: &Path, prompt_tokens: &[u32]) -> ForgeCoreGenerationRun {
    let registry = DeviceRegistry::discover();
    let execution = ExecutionConfig::new(DeviceSelection::Cpu, 1, None, false)
        .expect("CPU execution configuration");
    let model = ForgeCoreModel::load_for_execution(path, execution, &registry)
        .expect("ForgeCore CPU model must load");
    let gguf = parse_gguf_file(path).expect("fixture GGUF must parse");
    let tokenizer = RamforgeTokenizer::from_gguf(&gguf).expect("fixture tokenizer must load");
    let prompt_tokens = prompt_tokens.to_vec();
    assert!(
        !prompt_tokens.is_empty(),
        "fixture prompt must produce tokens"
    );

    let mut session = model
        .open_selected_context()
        .expect("ForgeCore CPU context must open");
    let vocab_size = session.vocab_size() as usize;

    assert!(
        prompt_tokens.iter().all(|id| (*id as usize) < vocab_size),
        "every prompt token must be inside the ForgeCore vocabulary: {prompt_tokens:?} / {vocab_size}"
    );
    assert!(
        prompt_tokens.len() <= session.context_length() as usize,
        "fixture prompt must fit the ForgeCore context"
    );

    let mut logits = session
        .decode_tokens(&prompt_tokens, 0)
        .expect("ForgeCore prompt prefill must decode");
    assert_eq!(logits.n_vocab(), vocab_size);
    assert!(logits.values().iter().all(|value| value.is_finite()));
    let mut logits_history = vec![logits.values().to_vec()];
    let sampler = Sampler::new(0.0, None, None);
    let mut generated_token_ids = Vec::new();

    for _ in 0..MAX_GENERATED_TOKENS {
        let next_token = sampler.sample(logits.values());
        assert!(
            (next_token as usize) < vocab_size,
            "every sampled token must be inside the ForgeCore vocabulary: {next_token} / {vocab_size}"
        );
        // Match the production ForgeCore CLI path: EOS stops generation before
        // it is appended or decoded as a new KV step.
        if Some(next_token) == tokenizer.eos_id {
            break;
        }
        generated_token_ids.push(next_token);

        if generated_token_ids.len() >= MAX_GENERATED_TOKENS {
            break;
        }
        let next_position = prompt_tokens
            .len()
            .checked_add(generated_token_ids.len())
            .and_then(|length| length.checked_sub(1))
            .expect("next token position must not overflow");
        let next_position = u32::try_from(next_position)
            .expect("next token position must fit ForgeCore's position type");
        logits = session
            .decode_token(next_token, next_position)
            .expect("ForgeCore one-step decode must succeed");
        assert_eq!(logits.n_vocab(), vocab_size);
        assert!(logits.values().iter().all(|value| value.is_finite()));
        logits_history.push(logits.values().to_vec());
    }

    ForgeCoreGenerationRun {
        prompt_tokens,
        generated_token_ids,
        logits: logits_history,
    }
}

#[test]
fn forgecore_cpu_generation_is_repeatable_with_compatible_bos_eos_metadata() {
    let Some(path) = fixture() else {
        return;
    };

    // Inspect the actual safe ForgeCore tokenizer metadata and the actual
    // RAMforge GGUF metadata before exercising generation. This deliberately
    // verifies only the BOS/EOS path used below, not full tokenizer parity.
    let gguf = parse_gguf_file(&path).expect("fixture GGUF must parse");
    let ramforge_tokenizer =
        RamforgeTokenizer::from_gguf(&gguf).expect("fixture tokenizer must load");
    let native_model = NativeModel::load(&path).expect("fixture model must load natively");
    let native_tokenizer =
        NativeTokenizer::open(&native_model).expect("native tokenizer metadata must open");
    let native_special = native_tokenizer.special_tokens();

    assert_eq!(
        native_tokenizer.n_vocab() as usize,
        ramforge_tokenizer.vocab_size(),
        "the two tokenizer APIs must expose the same observed vocabulary length"
    );
    assert_eq!(
        native_tokenizer.adds_bos(),
        ramforge_tokenizer.add_bos,
        "BOS insertion behavior must agree for the exercised path"
    );
    assert_eq!(
        native_tokenizer.adds_eos(),
        ramforge_tokenizer.add_eos,
        "EOS insertion behavior must agree for the exercised path"
    );

    let bos_id = ramforge_tokenizer
        .bos_id
        .expect("fixture must expose a RAMforge BOS id");
    let eos_id = ramforge_tokenizer
        .eos_id
        .expect("fixture must expose a RAMforge EOS id");
    assert_eq!(
        gguf
            .get_metadata("tokenizer.ggml.bos_token_id")
            .and_then(|value| value.as_u32()),
        Some(bos_id),
        "RAMforge BOS id must come from the fixture metadata"
    );
    assert_eq!(
        gguf
            .get_metadata("tokenizer.ggml.eos_token_id")
            .and_then(|value| value.as_u32()),
        Some(eos_id),
        "RAMforge EOS id must come from the fixture metadata"
    );
    assert_eq!(native_special.bos, Some(bos_id));
    assert_eq!(native_special.eos, Some(eos_id));
    assert!(native_tokenizer.is_control(bos_id).expect("BOS control query"));
    assert!(native_tokenizer.is_control(eos_id).expect("EOS control query"));
    assert!(native_tokenizer.is_eog(eos_id).expect("EOS end-of-generation query"));
    assert_eq!(
        ramforge_tokenizer.token_types.get(bos_id as usize),
        Some(&TokenType::Control)
    );
    assert_eq!(
        ramforge_tokenizer.token_types.get(eos_id as usize),
        Some(&TokenType::Control)
    );
    for id in [bos_id, eos_id] {
        let native_text = native_tokenizer
            .token_text(id)
            .expect("native special-token text");
        let ramforge_text = ramforge_tokenizer
            .tokens
            .get(id as usize)
            .expect("RAMforge special-token text");
        assert_eq!(
            native_text.as_str(),
            ramforge_text.as_str(),
            "special-token text for id {id}"
        );
    }

    // The default decode fixture contains ordinary whole-token pieces but no
    // character-level or byte-fallback pieces. Its generic RAMforge text
    // encoding path therefore cannot round-trip this fixture token because
    // the GGUF also carries a merges array. Use the verified vocabulary ID
    // directly here; this test does not claim full text-tokenizer parity.
    let prompt_piece_id = ramforge_tokenizer
        .tokens
        .iter()
        .position(|token| token == GENERATION_PROMPT)
        .map(|id| id as u32)
        .expect("fixture must expose the fixed prompt as one token");
    assert_eq!(
        ramforge_tokenizer.token_types.get(prompt_piece_id as usize),
        Some(&TokenType::Normal),
        "the fixed prompt must use an ordinary fixture token"
    );
    let prompt_tokens = if ramforge_tokenizer.add_bos {
        vec![bos_id, prompt_piece_id]
    } else {
        vec![prompt_piece_id]
    };
    assert_eq!(prompt_tokens.first().copied(), Some(bos_id));
    assert_eq!(prompt_tokens.last().copied(), Some(prompt_piece_id));
    assert_eq!(prompt_tokens.iter().filter(|id| **id == bos_id).count(), 1);
    assert!(
        !prompt_tokens.contains(&eos_id),
        "the fixture does not append EOS to this prompt because add_eos is false"
    );

    let first = run_forgecore_generation_once(&path, &prompt_tokens);
    let second = run_forgecore_generation_once(&path, &prompt_tokens);
    assert_eq!(first.prompt_tokens, prompt_tokens);
    assert_eq!(second.prompt_tokens, prompt_tokens);
    assert!(
        first.logits.len() >= 2,
        "the fixed fixture prompt must exercise at least one incremental decode"
    );
    assert_eq!(
        first.logits, second.logits,
        "the prompt prefill and repeated one-step logits must be identical"
    );
    assert_eq!(
        first.generated_token_ids, second.generated_token_ids,
        "greedy ForgeCore generation must produce the same token sequence twice"
    );
    assert!(
        first
            .generated_token_ids
            .iter()
            .all(|id| (*id as usize) < native_tokenizer.n_vocab() as usize),
        "every accepted generated token must be in the native vocabulary"
    );
}

#[test]
fn forgecore_ordinary_token_metadata_matches_ramforge_for_fixture() {
    let Some(path) = fixture() else {
        return;
    };

    // The default decode fixture cannot support general text-encoding parity:
    // it has whole-token pieces but no byte fallback, and its merges metadata
    // selects RAMforge's BPE path. This test intentionally compares only the
    // metadata for one verified ordinary vocabulary entry.
    let gguf = parse_gguf_file(&path).expect("fixture GGUF must parse");
    let ramforge_tokenizer =
        RamforgeTokenizer::from_gguf(&gguf).expect("fixture tokenizer must load");
    let ordinary_id = ramforge_tokenizer
        .tokens
        .iter()
        .position(|token| token == GENERATION_PROMPT)
        .map(|id| id as u32)
        .expect("fixture must expose the ordinary prompt token");
    let ramforge_text = ramforge_tokenizer
        .tokens
        .get(ordinary_id as usize)
        .expect("RAMforge ordinary token text");

    let native_model = NativeModel::load(&path).expect("fixture model must load natively");
    let native_tokenizer =
        NativeTokenizer::open(&native_model).expect("native tokenizer metadata must open");
    assert_eq!(
        native_tokenizer
            .token_text(ordinary_id)
            .expect("native ordinary token text"),
        ramforge_text.as_str(),
        "native and RAMforge ordinary token text must match"
    );
    assert!(
        native_tokenizer
            .token_attr(ordinary_id)
            .expect("native ordinary token attributes")
            .contains(TokenAttr::NORMAL),
        "native prompt token must be ordinary"
    );
    assert!(!native_tokenizer
        .is_control(ordinary_id)
        .expect("native ordinary control query"));
    assert!(!native_tokenizer
        .is_eog(ordinary_id)
        .expect("native ordinary EOG query"));
}

#[test]
fn forgecore_boundary_keeps_native_ffi_out_of_ramforge() {
    let manifest = include_str!("../Cargo.toml");
    let adapter = include_str!("../src/forgecore.rs");
    assert!(manifest.contains("forge-core"));
    assert!(!manifest.contains("forge-sys"));
    assert!(!adapter.contains("forge_sys"));
    assert!(!adapter.contains("extern \"C\""));
}
