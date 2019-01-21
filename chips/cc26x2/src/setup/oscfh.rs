/*
 * Copyright (c) 2015, Texas Instruments Incorporated - http://www.ti.com/
 * All rights reserved.
 *
 * Redistribution and use in source and binary forms, with or without
 * modification, are permitted provided that the following conditions
 * are met:
 * 1. Redistributions of source code must retain the above copyright
 *    notice, this list of conditions and the following disclaimer.
 * 2. Redistributions in binary form must reproduce the above copyright
 *    notice, this list of conditions and the following disclaimer in the
 *    documentation and/or other materials provided with the distribution.
 * 3. Neither the name of the copyright holder nor the names of its
 *    contributors may be used to endorse or promote products derived
 *    from this software without specific prior written permission.
 *
 * THIS SOFTWARE IS PROVIDED BY THE COPYRIGHT HOLDERS AND CONTRIBUTORS
 * ``AS IS'' AND ANY EXPRESS OR IMPLIED WARRANTIES, INCLUDING, BUT NOT
 * LIMITED TO, THE IMPLIED WARRANTIES OF MERCHANTABILITY AND FITNESS
 * FOR A PARTICULAR PURPOSE ARE DISCLAIMED.  IN NO EVENT SHALL THE
 * COPYRIGHT HOLDER OR CONTRIBUTORS BE LIABLE FOR ANY DIRECT, INDIRECT,
 * INCIDENTAL, SPECIAL, EXEMPLARY, OR CONSEQUENTIAL DAMAGES
 * (INCLUDING, BUT NOT LIMITED TO, PROCUREMENT OF SUBSTITUTE GOODS OR
 * SERVICES; LOSS OF USE, DATA, OR PROFITS; OR BUSINESS INTERRUPTION)
 * HOWEVER CAUSED AND ON ANY THEORY OF LIABILITY, WHETHER IN CONTRACT,
 * STRICT LIABILITY, OR TORT (INCLUDING NEGLIGENCE OR OTHERWISE)
 * ARISING IN ANY WAY OUT OF THE USE OF THIS SOFTWARE, EVEN IF ADVISED
 * OF THE POSSIBILITY OF SUCH DAMAGE.
*/

// Clock switching and source select code from Texas Instruments
// The registers and fields are undefined in the technical reference
// manual necesistating this component until it is revealed to the world.

use rom;
use setup::ddi;

#[allow(non_snake_case)]
pub unsafe extern "C" fn clock_source_set(ui32src_clk: u32, ui32osc: u32) {
    if ui32src_clk & 0x1u32 != 0 {
        // ui32Base, ui32Reg, ui32Mask, ui32Shift, ui32Data
        ddi::ddi16bitfield_write(0x400ca000u32, 0x0u32, 0x1u32, 0u32, ui32osc as (u16));
    }
    if ui32src_clk & 0x4u32 != 0 {
        ddi::ddi16bitfield_write(0x400ca000u32, 0x0u32, 0xcu32, 2u32, ui32osc as (u16));
    }
}

pub unsafe extern "C" fn clock_source_get(ui32src_clk: u32) -> u32 {
    let ui32clock_source: u32;
    if ui32src_clk == 0x4u32 {
        ui32clock_source =
            ddi::ddi16bitfield_read(0x400ca000u32, 0x3cu32, 0x60000000u32, 29u32) as (u32);
    } else {
        ui32clock_source =
            ddi::ddi16bitfield_read(0x400ca000u32, 0x3cu32, 0x10000000u32, 28u32) as (u32);
    }
    ui32clock_source
}

#[allow(unused)]
unsafe fn source_ready() -> bool {
    (if ddi::ddi16bitfield_read(0x400ca000u32, 0x3cu32, 0x1u32, 0u32) != 0 {
        1i32
    } else {
        0i32
    }) != 0
}

pub unsafe fn source_switch() {
    (rom::HAPI.hf_source_safe_switch)();
}
