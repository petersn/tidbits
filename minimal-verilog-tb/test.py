import cocotb
from cocotb.clock import Clock
from cocotb.triggers import ClockCycles

@cocotb.test()
async def test_main(dut):
    clock = Clock(dut.clk, 10, units="us")
    cocotb.start_soon(clock.start())

    # Assert the reset line for three cycles.
    dut.rst_n.value = 0
    await ClockCycles(dut.clk, 3)
    dut.rst_n.value = 1

    # Collect 500 steps of LFSR output.
    bits = []
    for _ in range(500):
        await ClockCycles(dut.clk, 1)
        bits.append(dut.uo_out.value[7])
    print("LFSR outputs:", bits)

    # Assert that we're full period.
    assert bits[:127] == bits[127:254]
    for i in range(4, 127):
        assert bits[:i] != bits[i:2*i]

