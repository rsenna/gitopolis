use std::{
    fmt::Write,
    hint::black_box,
    rc::Rc,
    sync::{Arc, atomic::AtomicBool},
};
use tango_bench::{IntoBenchmarks, benchmark_fn, tango_benchmarks, tango_main};

fn bench_command(
	name: impl Into<String>,
	command: impl Into<String> + Clone,
	stack: Stack,
	engine: EngineState,
) -> impl IntoBenchmarks {
	let commands = Spanned {
		span: Span::unknown(),
		item: command.into(),
	};
	[benchmark_fn(name, move |b| {
		let commands = commands.clone();
		let stack = stack.clone();
		let engine = engine.clone();
		b.iter(move || {
			let mut stack = stack.clone();
			let mut engine = engine.clone();
			#[allow(clippy::unit_arg)]
			black_box(
				evaluate_commands(
					&commands,
					&mut engine,
					&mut stack,
					PipelineData::empty(),
					Default::default(),
				)
					.unwrap(),
			);
		})
	})]
}

tango_benchmarks!(
    bench_load_standard_lib(),
    bench_load_use_standard_lib(),
    // Data types
    // Record
    bench_record_create(1),
    bench_record_create(10),
    bench_record_create(100),
    bench_record_create(1_000),
    bench_record_flat_access(1),
    bench_record_flat_access(10),
    bench_record_flat_access(100),
    bench_record_flat_access(1_000),
    bench_record_nested_access(1),
    bench_record_nested_access(2),
    bench_record_nested_access(4),
    bench_record_nested_access(8),
    bench_record_nested_access(16),
    bench_record_nested_access(32),
    bench_record_nested_access(64),
    bench_record_nested_access(128),
    bench_record_insert(1, 1),
    bench_record_insert(10, 1),
    bench_record_insert(100, 1),
    bench_record_insert(1000, 1),
    bench_record_insert(1, 10),
    bench_record_insert(10, 10),
    bench_record_insert(100, 10),
    bench_record_insert(1000, 10),
    // Table
    bench_table_create(1),
    bench_table_create(10),
    bench_table_create(100),
    bench_table_create(1_000),
    bench_table_get(1),
    bench_table_get(10),
    bench_table_get(100),
    bench_table_get(1_000),
    bench_table_select(1),
    bench_table_select(10),
    bench_table_select(100),
    bench_table_select(1_000),
    bench_table_insert_row(1, 1),
    bench_table_insert_row(10, 1),
    bench_table_insert_row(100, 1),
    bench_table_insert_row(1000, 1),
    bench_table_insert_row(1, 10),
    bench_table_insert_row(10, 10),
    bench_table_insert_row(100, 10),
    bench_table_insert_row(1000, 10),
    bench_table_insert_col(1, 1),
    bench_table_insert_col(10, 1),
    bench_table_insert_col(100, 1),
    bench_table_insert_col(1000, 1),
    bench_table_insert_col(1, 10),
    bench_table_insert_col(10, 10),
    bench_table_insert_col(100, 10),
    bench_table_insert_col(1000, 10),
    // Eval
    // Interleave
    bench_eval_interleave(100),
    bench_eval_interleave(1_000),
    bench_eval_interleave(10_000),
    bench_eval_interleave_with_interrupt(100),
    bench_eval_interleave_with_interrupt(1_000),
    bench_eval_interleave_with_interrupt(10_000),
    // For
    bench_eval_for(1),
    bench_eval_for(10),
    bench_eval_for(100),
    bench_eval_for(1_000),
    bench_eval_for(10_000),
    // Each
    bench_eval_each(1),
    bench_eval_each(10),
    bench_eval_each(100),
    bench_eval_each(1_000),
    bench_eval_each(10_000),
    // Par-Each
    bench_eval_par_each(1),
    bench_eval_par_each(10),
    bench_eval_par_each(100),
    bench_eval_par_each(1_000),
    bench_eval_par_each(10_000),
    // Config
    bench_eval_default_config(),
    // Env
    bench_eval_default_env(),
    // Encode
    // Json
    encode_json(100, 5),
    encode_json(10000, 15),
    // MsgPack
    encode_msgpack(100, 5),
    encode_msgpack(10000, 15),
    // Decode
    // Json
    decode_json(100, 5),
    decode_json(10000, 15),
    // MsgPack
    decode_msgpack(100, 5),
    decode_msgpack(10000, 15)
);

tango_main!();
