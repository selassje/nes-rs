mod common;
use common::run_simple_short_test;

#[test]
fn cli_latency() {
    run_simple_short_test("tests/cpu_interrupts_v2/1-cli_latency.nes");
}

#[test]
fn nmi_and_brk() {
    run_simple_short_test("tests/cpu_interrupts_v2/2-nmi_and_brk.nes");
}
#[test]
fn nmi_and_irq() {
    run_simple_short_test("tests/cpu_interrupts_v2/3-nmi_and_irq.nes");
}
#[test]
fn irq_and_dma() {
    run_simple_short_test("tests/cpu_interrupts_v2/4-irq_and_dma.nes");
}
#[ignore]
#[test]
fn branch_delay_irq() {
    run_simple_short_test("tests/cpu_interrupts_v2/5-branch_delay_irq.nes");
}
