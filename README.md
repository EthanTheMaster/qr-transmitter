# What is this?
This is a toy project written in Rust to showcase fountain codes namely LT Codes. This program works by supplying a file to transmit. A window will then flash QR codes like a fountain spouting water, and the [receiver program](https://github.com/EthanTheMaster/qr-receiver) will capture these QR codes as fast as it can attempting to reconstruct the supplied file like a cup being filled with water hence the name "fountain code". [Here is a demo of the program in action] (https://www.youtube.com/watch?v=Cvwe5HSOPU0)

# How to Install
You will need to install `cargo` to install this Rust project. Inside the project directory, type into the terminal `cargo install --path .` to install the binary.

# Basic Usage
`qr_transmit [c=1.0] [delta=0.01] [packets=14] [fullscreen] file_path`
<br />
<br />
The degree distribution used for the LT Code is the [Robust Soliton Distribution (pg. 2)](http://web.nmsu.edu/~jkliewer/paper/PKF_Allerton06.pdf), and the parameters `c` and `delta` are the tuning parameters for this distribution as outlined in the linked PDF. 
<br/>
<br/>
_Luby [4] showed that, for a suitably chosen c (independent of k and δ), the decoder can recover the data from n = kβ = k+c·√k·ln2(k/δ)LT code  symbols, with  probability at least 1−δ. However, as pointed out in [6, p. 592], in practice c can be treated as a free parameter_
<br/>
<br/>
The `packets` parameter adjusts the number of packets encoded in a single QR code allowing for higher throughput for each scan. It is recommended that the `fullscreen` parameter be added and the `packets` parameter be tuned such that the biggest QR code can be displayed on the entire screen for maximum throughput.
