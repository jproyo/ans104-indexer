use crate::errors::ParseError;
use serde::{Deserialize, Serialize};
use std::vec::Vec;

type Result<T> = std::result::Result<T, ParseError>;

#[derive(Debug, Deserialize, Serialize, PartialEq)]
pub struct Tag {
    name: String,
    value: String,
}

pub struct TagsReader<'a> {
    buffer: &'a [u8],
    pos: usize,
}

impl<'a> TagsReader<'a> {
    pub fn deserialize(buffer: &'a [u8]) -> Result<Vec<Tag>> {
        TagsReader { buffer, pos: 0 }.read_tags()
    }

    fn skip_long(&mut self) {
        while self.buffer.get(self.pos).map_or(false, |&b| b & 0x80 != 0) {
            self.pos += 1;
        }
        // Move past the last byte that was part of the long
        if self.pos < self.buffer.len() {
            self.pos += 1;
        }
    }

    fn read_tags(&mut self) -> Result<Vec<Tag>> {
        let mut tags = Vec::new();

        while let Some(ref mut length) = self.read_long() {
            if length.is_negative() {
                *length = length.abs();
                self.skip_long()
            }

            for _ in 0..*length {
                let name = self.read_string()?;
                let value = self.read_string()?;
                tags.push(Tag { name, value });
            }
        }

        Ok(tags)
    }

    fn read_long(&mut self) -> Option<i64> {
        let mut number = 0i64;
        let mut shift = 0usize;

        while self.pos < self.buffer.len() {
            let byte = self.buffer[self.pos];
            self.pos += 1;

            number |= ((byte & 0x7f) as i64) << shift;
            shift += 7;

            if byte & 0x80 == 0 || shift >= 28 {
                return Some((number >> 1) ^ -(number & 1));
            }
        }

        None
    }

    fn read_string(&mut self) -> Result<String> {
        let length = self.read_long().ok_or_else(|| ParseError::ExpectedLong)?;

        if length < 0 || (self.pos + length as usize) > self.buffer.len() {
            return Err(ParseError::InvalidLengthString);
        }

        let start_pos = self.pos;
        self.pos += length as usize;

        let str = String::from_utf8(self.buffer[start_pos..self.pos].to_vec())?;
        Ok(str)
    }
}

#[cfg(test)]
mod tests {
    use std::collections::HashMap;

    use super::*;
    use crate::errors::ParseError;

    lazy_static::lazy_static! {
        static ref EXPECTED_TAGS: HashMap<&'static str, Vec<Tag>> = HashMap::from([
             ("A34IVHlwZRBtYW5pZmVzdBhDb250ZW50LVR5cGVGYXBwbGljYXRpb24veC5hcndlYXZlLW1hbmlmZXN0K2pzb24A"
     , vec![
        Tag {
            name: "Type".to_owned(),
            value: "manifest".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/x.arweave-manifest+json".to_owned(),
        },
    ]),
     ("ATwYQ29udGVudC1UeXBlIGFwcGxpY2F0aW9uL2pzb24A"
     , vec![
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYm5jM2FrNmt5d2tsc2R5d2s2dm00NWwyazV2NnVpdjNhYms0cnJuNWJsajdreDZnc2V2aRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreibnc3ak6kywklsdywk6vm45l2k5v6uiv3abk4rrn5blj7kx6gsevi".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
     },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZW1yZHJ5ZmJ6dmZqMmNybm50NHdpZDJ0anpqeWZrdm5teWdrY3ZrY2Q3djM1bGhwemx0eRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreiemrdryfbzvfj2crnnt4wid2tjzjyfkvnmygkcvkcd7v35lhpzlty".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYXV2cmM0NmZ3Z2JnM3ZpdWI0cnh5bXc1a3c3eDZ5eGR0a2dxZWNzYjQ1M2ptMjdkeGF4NBhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreiauvrc46fwgbg3viub4rxymw5kw7x6yxdtkgqecsb453jm27dxax4".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYXVtd3Y0enNveDd6d2dqc2RwbWdtbXhieDR5emFuaXVka21vbG1ib3FlZm1neHRtYTduYRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreiaumwv4zsox7zwgjsdpmgmmxbx4yzaniudkmolmboqefmgxtma7na".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("AUwYQ29udGVudC1UeXBlMHRleHQvaHRtbDsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "Content-Type".to_owned(),
            value: "text/html; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpaDViNm10cTdocXA3d3h1NWJna3V2aXRkNHM2bDY3NzRqaHZodmRudXJ1ZDJ0bDVlNDNlaRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreih5b6mtq7hqp7wxu5bgkuvitd4s6l6774jhvhvdnurud2tl5e43ei".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("AU4YQ29udGVudC1UeXBlMnRleHQvcGxhaW47IGNoYXJzZXQ9dXRmLTgA"
     , vec![
        Tag {
            name: "Content-Type".to_owned(),
            value: "text/plain; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZGR0N2plNzRqamloMmU1N3B6Zmh3Y3huaWNzNGFlM3I1czdjdG82ajVkMmgzYnZxNXl2dRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreiddt7je74jjih2e57pzfhwcxnics4ae3r5s7cto6j5d2h3bvq5yvu".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZmdsb2N2eGo2ZWt0ZWl1YWw3dzRoZmJ1eHVtd2hwNDdibWM2YXBnaTJvN2FhbWo1YW5rcRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreifglocvxj6ekteiual7w4hfbuxumwhp47bmc6apgi2o7aamj5ankq".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpaGl1amNubzNnYWt5aWxmbHRjZnRta3lrdmR4amVnYXFmYm5wdmxvZDN0bnBhN3J3aHl1bRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreihiujcno3gakyilfltcftmkykvdxjegaqfbnpvlod3tnpa7rwhyum".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZnNja3JiajJmYXVtZDVlcTR3ank0cmNkZXIyYjZsN2dvbmtnenpwdXZseGwyc28zaTRkdRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreifsckrbj2faumd5eq4wjy4rcder2b6l7gonkgzzpuvlxl2so3i4du".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpaHp0bmx1NmtnMm9yZjViNjRzYnBuaWFzYzZwZHQ0bWZycmM1bzNydzN5cHN4bWtvZG1rZRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreihztnlu6kg2orf5b64sbpniasc6pdt4mfrrc5o3rw3ypsxmkodmke".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZ3M2ZzM0Z2JlcGtldHUyNnR3bHduZXFuZTczenF2eDJhcGthN2VvM295cGV3emlzc3hiNBhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreigs6g34gbepketu26twlwneqne73zqvx2apka7eo3oypewzissxb4".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZjZlb2tneG02cmFscGptczNwbGZhYWN1dW5wc3h6c200b3BkN3NtYjQ0YW16ejNyM2duZRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreif6eokgxm6ralpjms3plfaacuunpsxzsm4opd7smb44amzz3r3gne".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZ2d6YmJuZWFrb2I0Zmk3cnduejN4d2JxY3Y1N3FsNTczYnJ4ZWZ6Y3U0MzJucGp4N3F3ZRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreiggzbbneakob4fi7rwnz3xwbqcv57ql573brxefzcu432npjx7qwe".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZmI2ankydmo0aXNvb3ZmMmRrZW96cnR4a3FwdHh6amN0bzRsa3NzaTc1M2V4aDYzbnpuZRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreifb6jy2vj4isoovf2dkeozrtxkqptxzjcto4lkssi753exh63nzne".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYWY1N25xejVzeW9ldHRpc29pNWxzZGQzM3JrMjVwdDJrb3N1bW15N21tMmIzanZycWZweRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreiaf57nqz5syoettisoi5lsdd33rk25pt2kosummy7mm2b3jvrqfpy".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZzc1cnFhZmdscGM2bmI2YmthejNrcHE0M2R2Y2F6d2hodXBxeGFiaW1pcTdwcW9sbmlnbRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreig75rqafglpc6nb6bkaz3kpq43dvcazwhhupqxabimiq7pqolnigm".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpaG9neXR5ZzY2NjIzem9ueXNhNmpmMnozMnlwanljdmtlZHdiamhvMjJvbmVmN295N2dxeRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreihogytyg66623zonysa6jf2z32ypjycvkedwbjho22onef7oy7gqy".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpaDVnMzNmczZsdzNvbGF5NGFvMnozbHFiZjZtNDN6cjUzbTM1cW1rdjR2YWRqeW1hMzM3dRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreih5g33fs6lw3olay4ao2z3lqbf6m43zr53m35qmkv4vadjyma337u".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZW9yeXgzdnNzaGVlZjVsbGszZmE3eHphZ3hoN201cTZ2dWJ5dnRwY2U3NnV5a3ljcWwzaRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreieoryx3vssheef5llk3fa7xzagxh7m5q6vubyvtpce76uykycql3i".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZjRtaWRkdTM1MmJ2ZzVlb2JsNWp3M2F4d3pjaXJkeGNqcmxycWwyNHJhdndsbmk3MjRxbRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreif4middu352bvg5eobl5jw3axwzcirdxcjrlrql24ravwlni724qm".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYTNzZW9qM3E2dnIzeWZ0b2Fqc3RqMnBnaW1kNXRha3l6ZTQyNDZjeTdqM2d6a3g3c3ZkbRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreia3seoj3q6vr3yftoajstj2pgimd5takyze4246cy7j3gzkx7svdm".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZG12N2U3dHM2ZmhxZmV6amQzMnA0djNhNjdocXdueHZ6dmF5b2l1YXAzdGhvZ3Fmb3B2NBhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreidmv7e7ts6fhqfezjd32p4v3a67hqwnxvzvayoiuap3thogqfopv4".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZ21yZXgyNHF5NXB2c3hpMmhvcWVmdTduaXZuZDZud3llNGo1cHBsbXByNWJ2eWFhbWZyNBhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreigmrex24qy5pvsxi2hoqefu7nivnd6nwye4j5pplmpr5bvyaamfr4".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZXNid212NnUzM2xtNHg1eDIyZ2R4cWc0Z3R4a2VqejNieWIzcHlrZnRneWt6NGJ2bGJ3YRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreiesbwmv6u33lm4x5x22gdxqg4gtxkejz3byb3pykftgykz4bvlbwa".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYXh4YTQ2cWFwa3ZrY2RzdGk1eTJwYWt3d2p4amluNWRmaGJycGNhNGRtYmY1eWNicGgzYRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreiaxxa46qapkvkcdsti5y2pakwwjxjin5dfhbrpca4dmbf5ycbph3a".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZm1sNnlpd3lqcXczeHhuN3Z0a2t3NWN5c2lubDMzZ29xd21ka3lrNDZkZmgyaHdwN2k3eRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreifml6yiwyjqw3xxn7vtkkw5cysinl33goqwmdkyk46dfh2hwp7i7y".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYml2anFoNnV6d3J4enZtenpneWJ1bHA0N2JzemJzbHFiMjdtczd0dGZ3MmxiNzQ2cjV0dRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreibivjqh6uzwrxzvmzzgybulp47bszbslqb27ms7ttfw2lb746r5tu".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZWNpam9wa3Zzdmt0cjNmcDU2ZG8yMjNiNnVibzMyamNkbW43bHhmdWw2ZGwzNTNsN2k3YRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreiecijopkvsvktr3fp56do223b6ubo32jcdmn7lxful6dl353l7i7a".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZXZveHd1anhxNmNjd3d4em5hbXQ3ZjV0Ym1meHdsaXV0bmZ2Y2o2ZmtleHM3dGpjYmtpNBhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreievoxwujxq6ccwwxznamt7f5tbmfxwliutnfvcj6fkexs7tjcbki4".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYXV1NDVoYnJ4aGJ4aDYycGxmb3drajJuNG90M2NoM21jZDd6aXhod3B1cWd5bjUzcXVsaRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreiauu45hbrxhbxh62plfowkj2n4ot3ch3mcd7zixhwpuqgyn53quli".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZ3I1aWlha3gyazVpY2ZubHNiYXF6N3VwaDJ6MjNrZmFraG1tcWt5d2QzZzRrbTJvYWNwNBhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreigr5iiakx2k5icfnlsbaqz7uph2z23kfakhmmqkywd3g4km2oacp4".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpaHVhZ2VramxmaHl5cmlleHF6dGhtemF2am1nbTcyc3RueXo3cGllZXpmcmlrdmZ4M2RydRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreihuagekjlfhyyriexqzthmzavjmgm72stnyz7pieezfrikvfx3dru".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpY3FwNHh6aXUybW11b3hjbWoyNXEydHduYXVlZHdnZGVtZHF4M2Z4cHlsNHpodzJ6YXRnbRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreicqp4xziu2mmuoxcmj25q2twnauedwgdemdqx3fxpyl4zhw2zatgm".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpaHcyd3ByazJ1cmRtbnh0cWpwdGl3NDVkZm5sZWlmbzMyaWNrazZzZXJ6NnZ3a3JvNTZteRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreihw2wprk2urdmnxtqjptiw45dfnleifo32ickk6serz6vwkro56my".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZGdvNmFuNnFra3Y0aWFhMmd2cTZ3M255NXE1NmpvY2tsdHl6am9hY3RodmZ2NTY2N2FsdRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreidgo6an6qkkv4iaa2gvq6w3ny5q56jockltyzjoacthvfv5667alu".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZnZubHNleWVuZG1xY3FrcDZqcGJkd3EydnhqZ2N3aXhidnBndmY0Zm5ybHNrY25zdHFmbRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreifvnlseyendmqcqkp6jpbdwq2vxjgcwixbvpgvf4fnrlskcnstqfm".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZTZpNXZyY2U0N2E2Y3hmam9tMnR1eTZrNWNlNWw1ZGh1em9jc2lqamt2NG5rbGhvdzdrcRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreie6i5vrce47a6cxfjom2tuy6k5ce5l5dhuzocsijjkv4nklhow7kq".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZXljd2I3bGFhYnRrd2Nxa3ZhZWRhNHA1N2NkYWJ5cm54azdnY3l6ZnhxYXp5N3JtNzQ0bRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreieycwb7laabtkwcqkvaeda4p57cdabyrnxk7gcyzfxqazy7rm744m".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYnhzeWdodXl4bXl4M3poeTMzeDd0emdrZHF2cTZkYnh5YzJjaHJxMjNkdXJ5eDN1N2JuZRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreibxsyghuyxmyx3zhy33x7tzgkdqvq6dbxyc2chrq23duryx3u7bne".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZzZiaWJna3ptanZ6NHl1NTJ2aDJrbHZkbzcza3FhN2NjeW80cWR3cjVsemdvd2FxZ3dncRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreig6bibgkzmjvz4yu52vh2klvdo73kqa7ccyo4qdwr5lzgowaqgwgq".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpaGZ0b29pbmFkNWtwcWlkc2sza3ZjZWlsNGh0eWZnbjRhbXRlZmJ6dTY0eXhwdHQ0d3JheRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreihftooinad5kpqidsk3kvceil4htyfgn4amtefbzu64yxptt4wray".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYnVvdW9hbHAzbGUycG82YXBheTZlYjZ3bWQ2Zmc2NzVkNHk1cXQ3YjRkY3A2aGw3eWl5ZRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreibuouoalp3le2po6apay6eb6wmd6fg675d4y5qt7b4dcp6hl7yiye".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZ3Q2N2lwY3I2YXZ3bGU3dnd4dzVkaXluN3lrZ3M2ZWxnYWhheHdyYXhhdW5mcnV2cHhjZRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreigt67ipcr6avwle7vwxw5diyn7ykgs6elgahaxwraxaunfruvpxce".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZnU3a2hrdHFubW9jZGphbWxlbnl0dGprZzd3NXNscXNwcmZ6aTd1eHlyeGR2c2FpcXdpYRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreifu7khktqnmocdjamlenyttjkg7w5slqsprfzi7uxyrxdvsaiqwia".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYTQ3Y2FoMm41N3N6Mjc2b3J0aDZtdDJlcm1oa3RmNHhjZmhmeDV3NmZpNHp3cmt5Nm9rNBhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreia47cah2n57sz276orth6mt2ermhktf4xcfhfx5w6fi4zwrky6ok4".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZXh5dzV2a2F2NnQ3eWtrcWhqdjN4Mm43cWN1M2ltZXZrdTNmdWd2ZzZxbjJ2YTJhYmhkeRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreiexyw5vkav6t7ykkqhjv3x2n7qcu3imevku3fugvg6qn2va2abhdy".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZ3Z3aHV6bWk1YmhpaHp2eGtkcWtsZ2M2djNtbzVveDV1dXF2bGhic2o2dnN5MnNleHA3bRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreigvwhuzmi5bhihzvxkdqklgc6v3mo5ox5uuqvlhbsj6vsy2sexp7m".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZ211bjJwYXFyb2NubGpvN3E3NXB3aGhtdGZlc3piNXp0bmtqa21ib3d6M3hiamIybHl3ZRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreigmun2paqrocnljo7q75pwhhmtfeszb5ztnkjkmbowz3xbjb2lywe".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZDdkMmVoc2txZ2ZsemFwN2V2a2d3Z2VjdXBjeGE0Z210cTdpZHlvNjVqN2I0eXN4dWlvbRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreid7d2ehskqgflzap7evkgwgecupcxa4gmtq7idyo65j7b4ysxuiom".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpaHJybDR2MnlsdmdwbDZxdTZ1YnJkY3E0aDNxcDZ2eHM3eHAzeGh2cW1nYWdxdWp3d3dubRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreihrrl4v2ylvgpl6qu6ubrdcq4h3qp6vxs7xp3xhvqmgagqujwwwnm".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYjZoencyZng0emljNzQ0YnJydWd4ZWI1amx6YmZ4ZXBuY2d0aWp5M2xsZGo2cTJuNm15YRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreib6hzw2fx4zic744brrugxeb5jlzbfxepncgtijy3lldj6q2n6mya".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYm16cmg3ZnJnN2FsdDVnamNqZHZ3ZmFlYW9xN3I2cmI1dmhha2tkMnFwZXd1NnhheGFhcRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreibmzrh7frg7alt5gjcjdvwfaeaoq7r6rb5vhakkd2qpewu6xaxaaq".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYW1pcmdnbndnZjJzYWg0NHc0Z3kyNGZxYzV1dWhmZWUyZWY3Y2lyYmF6bmtmdnRwNXJpNBhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreiamirggnwgf2sah44w4gy24fqc5uuhfee2ef7cirbaznkfvtp5ri4".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZnBuNzY0ZTN5NjZza3Fua2hjNmRlaGpvZG1oazVvcHllN2Fkc25wZmh0MnpobG94b2NnYRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreifpn764e3y66skqnkhc6dehjodmhk5opye7adsnpfht2zhloxocga".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZTY1YnhnYnE1d3AyNXV4YnJscmVoenpiYTNhN2pyN2Rqb3NtNjR2Z2Nxd29qeWRncDV4ZRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreie65bxgbq5wp25uxbrlrehzzba3a7jr7djosm64vgcqwojydgp5xe".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYzdldGE3ZGt3Ymc1YjRha293ZGZrejZiZWNuNDJlenpqa3FyY2Zxa2x4cG95YmQyb3dpeRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreic7eta7dkwbg5b4akowdfkz6becn42ezzjkqrcfqklxpoybd2owiy".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZWJ3NTU2dGxlb2RjYmd5b29leWFzanpsZjVoZ3ppbmVvbHBxNndudjJrdGp6aGxubnZkYRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreiebw556tleodcbgyooeyasjzlf5hgzineolpq6wnv2ktjzhlnnvda".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZml4d3g1azdrN3NtY3RtZHB2Y2M2cm1kcDN3Z2NkZXppa3pyZXl5cnJ6dGs0MmhrZ3VrZRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreifixwx5k7k7smctmdpvcc6rmdp3wgcdezikzreyyrrztk42hkguke".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZ3d1czMya2tkeXAyYXh5enJ6Z3J4YTJscHJxcjJzeG1nc3lxMjNsM2k1NHhqdHc2M211YRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreigwus32kkdyp2axyzrzgrxa2lprqr2sxmgsyq23l3i54xjtw63mua".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYWNxcXNnY2o0Nnh2bWE3YmhzaWQ1cGFnaG15Z3ByMjd1bnZ4bHoybHNzcDV4cXFqYjVzZRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreiacqqsgcj46xvma7bhsid5paghmygpr27unvxlz2lssp5xqqjb5se".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYjduYXhndng3MndqNmU2dnl0aGJmY3Mya21jbWJmeXlpcHplcGFiZnZuaGFsemVma3EyZRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreib7naxgvx72wj6e6vythbfcs2kmcmbfyyipzepabfvnhalzefkq2e".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZ2k0YzNxNWRydWtyYzJhcXltajZpcHozamo2dGI1ZWZnYnBqNnZzamtwcTRoMjZvY2U3dRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreigi4c3q5drukrc2aqymj6ipz3jj6tb5efgbpj6vsjkpq4h26oce7u".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYjY3cW92a2R0MzNkZHZ6N2tiam4zbXY3Z2JsbzVha2FrZmtmdGd3eTd1dW1nY2dnd2pvZRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreib67qovkdt33ddvz7kbjn3mv7gblo5akakfkftgwy7uumgcggwjoe".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZzZjcHp1Nzdld3VkbHhwYm5id25kYWtpemN0bnFwbWFmZjZha2Z6c25mNzVxZjN6dTd1eRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreig6cpzu77ewudlxpbnbwndakizctnqpmaff6akfzsnf75qf3zu7uy".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpZ2Eyamo0NXZ6aHdxdm9wdHppeGJlcW9penRkNWt3YmhrZ2pwcm03M21haGVrbG80Mm8zbRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreiga2jj45vzhwqvoptzixbeqoiztd5kwbhkgjprm73maheklo42o3m".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("A-YBEklQRlMtSGFzaHZiYWZrcmVpYWVjZjZ4dWF1ZG5jdHV2aGVqaW9oY2M2amt0b2xzY2Fka25zeHdhNGU1YnA0d3RyNWJwdRhDb250ZW50LVR5cGU-YXBwbGljYXRpb24vanNvbjsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "IPFS-Hash".to_owned(),
            value: "bafkreiaecf6xuaudnctuvhejiohcc6jktolscadknsxwa4e5bp4wtr5bpu".to_owned(),
        },
        Tag {
            name: "Content-Type".to_owned(),
            value: "application/json; charset=utf-8".to_owned(),
        },
    ]),
     ("AVgYQ29udGVudC1UeXBlPHRleHQvamF2YXNjcmlwdDsgY2hhcnNldD11dGYtOAA"
     , vec![
        Tag {
            name: "Content-Type".to_owned(),
            value: "text/javascript; charset=utf-8".to_owned(),
        },
        ])]);
    }

    #[test]
    fn test_deserialize_tags() {
        let fixtures = std::fs::read_dir("tests/fixtures").unwrap();
        for f in fixtures {
            let file = f.unwrap();
            let buffer = std::fs::read(file.path()).unwrap();
            let expected_tags = &EXPECTED_TAGS[file.file_name().to_str().unwrap()];
            let result = TagsReader::deserialize(&buffer);
            assert!(result.is_ok());
            let tags = result.unwrap();
            assert_eq!(&tags, expected_tags);
        }
    }

    #[test]
    fn test_deserialize_tags_invalid_length() {
        let buffer = b"\x02\x00\x03\x00\x04\x00\x05\x00\x06\x00\x07\x00";
        let result = TagsReader::deserialize(buffer);
        assert!(result.is_err());
        let error = result.err().unwrap();
        assert_eq!(error, ParseError::InvalidLengthString);
    }
}
