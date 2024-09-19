[![Build](https://github.com/dchansen06/mjpg-streamer/actions/workflows/build.yml/badge.svg)](https://github.com/dchansen06/mjpg-streamer/actions/workflows/build.yml)
# mjpg-streamer
A rust-based MJPG streamer with an eye towards compatibility with OctoPrint

# License information
This program is free software: you can redistribute it and/or modify it under the terms of the GNU General Public License as published by the Free Software Foundation, either version 3 of the License, or (at your option) any later version.

This program is distributed in the hope that it will be useful, but WITHOUT ANY WARRANTY; without even the implied warranty of MERCHANTABILITY or FITNESS FOR A PARTICULAR PURPOSE. See the GNU General Public License for more details.

You should have received a copy of the GNU General Public License along with this program. If not, see <https://www.gnu.org/licenses/>.

# Stability
This is not yet stable or finalized software

Please report any bugs or security issues

# Installation
Install the required dependencies with `sudo apt-get install libopencv-dev libclang-dev --no-install-suggests --no-install-recommends` and then install [cargo](https://doc.rust-lang.org/cargo/getting-started/installation.html).

Install the program with `cargo install --git https://github.com/dchansen06/mjpg-streamer/` and then run the binary (generally in `~/.cargo/bin`) with any desired flags.

Once completed the installation process should become a lot simpler, but this program is still well in the pre-alpha release phase, so USE WITH CAUTION AT YOUR OWN RISK.

## Security
Use at your own risk, if someone can ping your IP they can watch the stream. The apikey feature is not a replacement, it only adds a layer of [obfustication](https://en.wikipedia.org/wiki/Security_through_obscurity#Criticism).

# To-Do
* Divide into more discrete functions
* Setup a testing suite
* Run a best-guess attempt at getting inside through the client-side
* Ask for external comment
