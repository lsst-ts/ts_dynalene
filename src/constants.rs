// This file is part of ts_dynalene.
//
// Developed for the Vera Rubin Observatory Systems.
// This product includes software developed by the LSST Project
// (https://www.lsst.org).
// See the COPYRIGHT file at the top-level directory of this distribution
// for details of code ownership.
//
// This program is free software: you can redistribute it and/or modify
// it under the terms of the GNU General Public License as published by
// the Free Software Foundation, either version 3 of the License, or
// (at your option) any later version.
//
// This program is distributed in the hope that it will be useful,
// but WITHOUT ANY WARRANTY; without even the implied warranty of
// MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE.  See the
// GNU General Public License for more details.
//
// You should have received a copy of the GNU General Public License
// along with this program.  If not, see <https://www.gnu.org/licenses/>.

pub const NUM_TEMPERATURE_HUB: usize = 3;
pub const NUM_TEMPERATURE_CHANNEL: usize = 8;

// For the addresses of the pressure transducers, see the
// svi_PS_Sensor_Clasification_in.vi in dynalene_system LabVIEW project.
pub const ADDRESSES_PRESSURE_TRANSDUCER_BUS_0: [u8; 16] =
    [5, 6, 7, 8, 9, 10, 11, 12, 13, 14, 20, 21, 22, 23, 24, 25];
pub const ADDRESSES_PRESSURE_TRANSDUCER_BUS_1: [u8; 2] = [1, 2];
pub const ADDRESSES_PRESSURE_TRANSDUCER_BUS_2: [u8; 2] = [3, 4];

pub const NUM_BUS_PRESSURE_TRANSDUCER: usize = 3;

// The number of bytes in the response of temperature hub.
pub const BYTES_RESPONSE_TEMPERATURE: usize = 112;

// The number of bytes in the response of pressure transducer.
pub const BYTES_RESPONSE_PRESSURE: usize = 20;
