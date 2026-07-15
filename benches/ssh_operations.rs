// SPDX-License-Identifier: MIT OR Apache-2.0
//! Benchmarks de operações do ssh-cli.

use criterion::{black_box, criterion_group, criterion_main, Criterion};
use ssh_cli::masking::mask;
use ssh_cli::paths::{normalizar_nfc, validate_and_normalize, validate_name};

fn bench_mascaramento(c: &mut Criterion) {
    c.bench_function("mascarar_short", |b| {
        b.iter(|| mask(black_box("curto")))
    });
    c.bench_function("mascarar_long", |b| {
        b.iter(|| mask(black_box("senha-secreta-muito-longa-aqui-123456")))
    });
    c.bench_function("mascarar_unicode", |b| {
        b.iter(|| mask(black_box("ação você está configuração Itaú")))
    });
}

fn bench_paths(c: &mut Criterion) {
    c.bench_function("validar_nome", |b| {
        b.iter(|| validate_name(black_box("meu-servidor-producao")))
    });
    c.bench_function("normalizar_nfc_nfd", |b| {
        b.iter(|| normalizar_nfc(black_box("cafe\u{0301}")))
    });
    c.bench_function("normalizar_nfc_noop", |b| {
        b.iter(|| normalizar_nfc(black_box("servidor")))
    });
    c.bench_function("validar_e_normalizar", |b| {
        b.iter(|| validate_and_normalize(black_box("meu-servidor")))
    });
}

criterion_group!(benches, bench_mascaramento, bench_paths);
criterion_main!(benches);
