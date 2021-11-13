use std::{convert::TryInto, time::Duration};

use tokio::io::{AsyncBufRead, AsyncReadExt};

pub const MAIL_TYPE_SIZE: usize = std::mem::size_of::<u8>();
pub const MAIL_PAYLOAD_SIZE: usize = std::mem::size_of::<u64>();

/// Mail is the structure of the data that can be exchanged between parent and
/// the child process.
///
/// The exchange format is based on `TLV` format, where T can be part of the data,
/// L is represented by the `size` and should be the first 4 bytes of the data that is read (hence
/// the total size is 64 bits) and V is represented by the `data` and MUST be of the same length as
/// mentioned in the `data` attribute.
///
/// Every IO message between child and parent process MUST be of `Mail` format, in case the message
/// fails to be parsed into `Mail` then the message should be dropped.
#[derive(Clone)]
pub struct Mail {
    pub typ: u8,
    pub size: u64,
    pub data: Vec<u8>,
}

impl Mail {
    pub fn as_bytes_vec(&self) -> Vec<u8> {
        let mut res = vec![self.typ];
        res.extend(self.size.to_be_bytes());
        res.extend(&self.data);

        res
    }

    pub async fn from_stream<T>(stream: &mut T) -> Result<Self, std::io::Error>
    where
        T: AsyncBufRead + Unpin,
    {
        let mut data: Vec<u8> = Vec::new();

        // Read from the stream
        let mut buffer: Vec<u8> = vec![0; 128];

        loop {
            let res = stream.read(&mut buffer).await;

            return match res {
                Ok(n) => {
                    // If nothing is read then the process has probably died
                    if n == 0 {
                        return Ok(Self { typ: 0, size: 0, data: buffer });
                    }

                    // Copy data to the local data store
                    data.append(&mut buffer[..n].to_vec());

                    // If more than or equal to 1 byte are read then setup then setup the type of the read
                    let typ: u8 = data[0];

                    // If length is 5 bytes or more then safe to parse the size of the data or else retry
                    if n < MAIL_TYPE_SIZE + MAIL_PAYLOAD_SIZE {
                        continue;
                    }
                    let payload_size = u64::from_be_bytes(
                        data[MAIL_TYPE_SIZE..MAIL_TYPE_SIZE + MAIL_PAYLOAD_SIZE]
                            .try_into()
                            .unwrap(),
                    );

                    // Read till the payload_size
                    if n < payload_size as usize + MAIL_TYPE_SIZE + MAIL_PAYLOAD_SIZE {
                        continue;
                    }
                    let payload = &data[MAIL_TYPE_SIZE + MAIL_PAYLOAD_SIZE..];

                    Ok(Self {
                        typ,
                        size: payload_size,
                        data: payload.to_vec(),
                    })
                }
                Err(e) => Err(e),
            };
        }
    }
}

pub mod data_type {
    pub const LOG: u8 = 0;
    pub const DATA: u8 = 1;
}
