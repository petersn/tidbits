module tt_um_main (
  input  wire [7:0] ui_in,    // Dedicated inputs - connected to the input switches
  output wire [7:0] uo_out,   // Dedicated outputs - connected to the 7 segment display
  input  wire [7:0] uio_in,   // IOs: Bidirectional Input path
  output wire [7:0] uio_out,  // IOs: Bidirectional Output path
  output wire [7:0] uio_oe,   // IOs: Bidirectional Enable path (active high: 0=input, 1=output)
  input  wire       ena,      // will go high when the design is enabled
  input  wire       clk,      // clock
  input  wire       rst_n     // reset_n - low to reset
);
  reg [7:0] lfsr;
  always @(posedge clk) begin
    if (!rst_n) begin
      lfsr <= 8'b0000_0001;
    end else begin
      lfsr <= {lfsr[6:0], lfsr[7] ^ lfsr[3] ^ lfsr[2]};
    end
  end
  // Expose the LFSR's output on an output wire.
  assign uo_out[0] = lfsr[7];

  // Make sure all outputs are driven.
  assign uo_out[7:1] = 7'h00;
  assign uio_out = 8'h00;
  assign uio_oe = 8'h00;
endmodule
