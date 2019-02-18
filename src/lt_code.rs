use rand::Rng;

//Rng State using Xor Shift PRNG
pub struct XorShift {
    state: u32
}

impl XorShift {
    pub fn from_seed(seed: u32) -> XorShift {
        XorShift {state: seed}
    }

    pub fn rand(&mut self) -> u32 {
        let mut x = self.state;
        x ^= x << 13;
        x ^= x >> 17;
        x ^= x << 5;

        self.state = x;
        return x;
    }
}

pub struct Block {
    pub index: u32,
    pub content: u8
}

//Packet to send
pub struct Packet {
    //Size of file in bytes
    pub total_size: u32,
    //Number of blocks inside the packet
    pub degree: u32,
    //RNG used to generate list of contained blocks
    pub rng_seed: u32,
    pub xor_block: Block,
}

pub struct Fountain<'a> {
    pub data: &'a [u8],
    //Probability distribution of degrees
    pub pdf: Vec<f64>
}

impl<'a> Fountain<'a> {
    //chunk_size, c, and delta are tuning parameters for the robust soliton degree distribution
    pub fn new(data: &[u8], chunk_size: u32, c: f64, delta: f64) -> Fountain {
        assert!(data.len() < std::u32::MAX as usize);
        assert!((chunk_size as usize) <= data.len());

        let n = chunk_size;
        //Use robust soliton distribution ... ith index gives probability of (i + 1)th degree
        let mut pdf: Vec<f64> = Vec::new();

        let S = c * (chunk_size as f64).powf(0.5) * ((chunk_size as f64) / delta).ln();
        for i in 0..data.len() {
            let deg = i+1;
            let soliton = if deg == 1 {
                1.0 / (chunk_size as f64)
            } else if deg <= chunk_size as usize {
                1.0/((deg * (deg-1)) as f64)
            } else {
                0.0
            };

            let t_i = if deg <= ((chunk_size as f64 / S) as usize - 1) {
                S/((deg * chunk_size as usize) as f64)
            } else if deg == (chunk_size as f64/S) as usize {
                S * (S/delta).ln() / (chunk_size as f64)
            } else {
                0.0
            };

            pdf.push(soliton + t_i);

        }

        //Normalize robust soliton pdf
        let sum: f64 = pdf.iter().fold(0.0, |acc, x| acc + x);
        for p in pdf.iter_mut() {
            *p /= sum;
        }

        println!("S: {}", S);
        println!("Total Size: {}", data.len());
        Fountain {data, pdf}
    }

    pub fn generate_packet(&self) -> Packet {
        //Generate degree
        let mut rng = rand::thread_rng();
        let x: f64 = rng.gen();
        let mut d = 1;

        //Sample from pdf
        let mut accumulator = 0.0;
        for p in self.pdf.iter() {
            accumulator += *p;

            if x <= accumulator {
                break;
            } else {
                d += 1;
            }
        }

        //Generate xor block of d blocks
        let seed: u32 = rng.gen();
        let mut xor_rng = XorShift::from_seed(seed);
        let mut block = Block{index: 0, content: 0};

        for _ in 0..d {
            let idx = (xor_rng.rand() as usize % self.data.len()) as u32;
            let content = self.data[idx as usize];

            block = Block { index: block.index ^ idx, content: block.content ^ content };
        }

        return Packet{
            total_size: self.data.len() as u32,
            degree: d,
            rng_seed: seed,
            xor_block: block,
        };
    }

    pub fn serialize_packet(packet: &Packet) -> [u8; 20] {
        //A packet has 20 bytes ... total_size(4 bytes) + degree(4 bytes) + rng_seed(4 bytes) + xor_block(4 + 1 bytes) + parity (3 bytes)
        /*
            Packet Structure:
                Bytes 0 - 3: total_size,
                Bytes 4 - 7: degree,
                Bytes 8 - 11: rng_seed,
                Bytes 12 - 16: xor_block
                    Bytes 12 - 15: index
                    Byte 15: content
                Bytes 17-19: parity
        */
        let mut bytes: [u8; 20] = [0; 20];
        //Encode the total_size
        bytes[0] = (packet.total_size >> 24 & 0xFF) as u8;
        bytes[1] = (packet.total_size >> 16 & 0xFF) as u8;
        bytes[2] = (packet.total_size >> 8 & 0xFF) as u8;
        bytes[3] = (packet.total_size & 0xFF) as u8;
        //Encode the degree
        bytes[4] = (packet.degree >> 24 & 0xFF) as u8;
        bytes[5] = (packet.degree >> 16 & 0xFF) as u8;
        bytes[6] = (packet.degree >> 8 & 0xFF) as u8;
        bytes[7] = (packet.degree & 0xFF) as u8;
        //Encode the rng state
        bytes[8] = (packet.rng_seed >> 24 & 0xFF) as u8;
        bytes[9] = (packet.rng_seed >> 16 & 0xFF) as u8;
        bytes[10] = (packet.rng_seed >> 8 & 0xFF) as u8;
        bytes[11] = (packet.rng_seed & 0xFF) as u8;
        //Encode the block
        bytes[12] = (packet.xor_block.index >> 24 & 0xFF) as u8;
        bytes[13] = (packet.xor_block.index >> 16 & 0xFF) as u8;
        bytes[14] = (packet.xor_block.index >> 8 & 0xFF) as u8;
        bytes[15] = (packet.xor_block.index & 0xFF) as u8;
        bytes[16] = packet.xor_block.content;

        //Make use of avalanche effect in rng
        let mut parity_rng = XorShift::from_seed(packet.rng_seed);
        let mut parity = parity_rng.rand();
        parity_rng = XorShift::from_seed(parity ^ packet.total_size);
        parity = parity_rng.rand();
        parity_rng = XorShift::from_seed(parity ^ packet.degree);
        parity = parity_rng.rand();
        parity_rng = XorShift::from_seed(parity ^ packet.xor_block.index);
        parity = parity_rng.rand();
        parity_rng = XorShift::from_seed(parity ^ (packet.xor_block.content as u32));
        parity = parity_rng.rand();

        bytes[17] = (parity >> 16 & 0xFF) as u8;
        bytes[18] = (parity >> 8 & 0xFF) as u8;
        bytes[19] = (parity & 0xFF) as u8;
        return bytes;
    }
}