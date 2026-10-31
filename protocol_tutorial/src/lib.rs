use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;

/// 与图解教程示例报文一致：
/// type=0x11, fc=51 73 B8, data1=55 B5, data2=93 65 34 12,
/// data3=0xAD (lo=5,hi=21), data4=AB 55 55
#[derive(Clone, Debug, Serialize, Deserialize)]
#[serde(default)] // 缺字段时用 Default（与图解示例一致），避免旧前端反序列化失败
pub struct ProtocolForm {
    pub msg_type: u8,
    pub fc_hex: String,
    pub data1_type_code: u8,
    pub data1_s1: bool,
    pub data1_s2: bool,
    pub data1_s3: bool,
    pub data1_code: u16,
    pub data2_feild_2: u8,
    pub data2_feild_1: u8,
    pub data2_feild_4: u8,
    pub data2_feild_3: u8,
    pub data2_sys_code: u16,
    /// data3: bit0~2
    pub data3_lo: u8,
    /// data3: bit3~7
    pub data3_hi: u8,
    /// data4: bit8~0 (9bit)
    pub data4_info_c: u16,
    /// data4: bit16~9 (8bit)
    pub data4_info_b: u8,
    /// data4: bit23~17 (7bit)
    pub data4_info_a: u8,
}

impl Default for ProtocolForm {
    fn default() -> Self {
        Self {
            msg_type: 0x11,
            fc_hex: "51 73 B8".to_string(),
            data1_type_code: 0b101,
            data1_s1: true,
            data1_s2: false,
            data1_s3: true,
            data1_code: 0x155,
            data2_feild_2: 0b1001,
            data2_feild_1: 0b0011,
            data2_feild_4: 0b0110,
            data2_feild_3: 0b0101,
            data2_sys_code: 0x1234,
            data3_lo: 5,  // bit0~2
            data3_hi: 21, // bit3~7 → 字节 0xAD
            data4_info_c: 0x155,
            data4_info_b: 0xAA,
            data4_info_a: 0x55,
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct PacketBuild {
    pub packet: Vec<u8>,
    pub host_data1_raw: u16,
    pub host_data2_raw: u32,
    pub host_data3_raw: u8,
    pub host_data4_raw: u32,
    pub data1_raw: u16,
    pub data2_raw: u32,
    pub data3_raw: u8,
    pub data4_be: [u8; 3],
}

#[derive(Clone, Debug, Serialize)]
pub struct ParsedPacket {
    pub msg_type: u8,
    pub len: u16,
    pub fc: Vec<u8>,
    pub data1_type_code: u8,
    pub data1_s1: bool,
    pub data1_s2: bool,
    pub data1_s3: bool,
    pub data1_code: u16,
    pub data2_feild_2: u8,
    pub data2_feild_1: u8,
    pub data2_feild_4: u8,
    pub data2_feild_3: u8,
    pub data2_sys_code: u16,
    pub data3_lo: u8,
    pub data3_hi: u8,
    pub data4_info_c: u16,
    pub data4_info_b: u8,
    pub data4_info_a: u8,
}

#[derive(Clone, Debug, Serialize)]
pub struct BuildResponse {
    pub packet_hex: String,
    pub bit_desc: String,
    pub server_struct: String,
    pub packet_len: usize,
}

#[derive(Clone, Debug, Serialize)]
pub struct SimulateFullResponse {
    pub tx_flow: String,
    pub client_mem: String,
    pub client_struct: String,
    pub teach_panel: String,
    pub verify: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct StepPreviewResponse {
    pub tx_flow: String,
    pub client_mem: String,
    pub teach_panel: String,
}

#[derive(Clone, Debug, Serialize)]
pub struct BitFinalizeResponse {
    pub client_mem: String,
    pub client_struct: String,
    pub teach_panel: String,
    pub verify: String,
}

pub fn random_form() -> ProtocolForm {
    let mut rng = rand::thread_rng();
    ProtocolForm {
        msg_type: rng.gen(),
        fc_hex: "81 83 24".to_string(),
        data1_type_code: rng.gen_range(0..=7),
        data1_s1: rng.gen_bool(0.5),
        data1_s2: rng.gen_bool(0.5),
        data1_s3: rng.gen_bool(0.5),
        data1_code: rng.gen_range(0..=1023),
        data2_feild_2: rng.gen_range(0..=15),
        data2_feild_1: rng.gen_range(0..=15),
        data2_feild_4: rng.gen_range(0..=15),
        data2_feild_3: rng.gen_range(0..=15),
        data2_sys_code: rng.gen(),
        data3_lo: rng.gen_range(0..=7),
        data3_hi: rng.gen_range(0..=31),
        data4_info_c: rng.gen_range(0..=0x1ff),
        data4_info_b: rng.gen_range(0..=0xff),
        data4_info_a: rng.gen_range(0..=0x7f),
    }
}

pub fn build_response(form: &ProtocolForm) -> Result<BuildResponse, String> {
    let pkt = build_packet(form)?;
    Ok(BuildResponse {
        packet_hex: to_hex(&pkt.packet),
        bit_desc: render_bit_layout(&pkt),
        server_struct: render_server_protocol_table(form, &pkt),
        packet_len: pkt.packet.len(),
    })
}

pub fn simulate_full_transfer(form: &ProtocolForm) -> Result<SimulateFullResponse, String> {
    let pkt = build_packet(form)?;
    let packet = &pkt.packet;
    let mut verify = format!("一次发送报文 HEX: {}\n", to_hex(packet));
    match parse_packet(packet) {
        Ok(p) => {
            verify.push_str(&render_parsed(&p));
            let round_ok = p.msg_type == form.msg_type
                && p.data1_code == form.data1_code
                && p.data1_type_code == form.data1_type_code
                && p.data1_s1 == form.data1_s1
                && p.data1_s2 == form.data1_s2
                && p.data1_s3 == form.data1_s3
                && p.data2_feild_1 == form.data2_feild_1
                && p.data2_feild_2 == form.data2_feild_2
                && p.data2_feild_3 == form.data2_feild_3
                && p.data2_feild_4 == form.data2_feild_4
                && p.data2_sys_code == form.data2_sys_code
                && p.data3_lo == (form.data3_lo & 0x7)
                && p.data3_hi == (form.data3_hi & 0x1f)
                && p.data4_info_a == (form.data4_info_a & 0x7f)
                && p.data4_info_b == form.data4_info_b
                && p.data4_info_c == (form.data4_info_c & 0x1ff);
            verify.push_str(if round_ok {
                "\n校验: PASS（解析字段与表单一致）"
            } else {
                "\n校验: FAIL（解析字段与表单不一致）"
            });
        }
        Err(e) => {
            verify.push_str(&format!("解析失败: {e}\n校验: FAIL"));
        }
    }
    Ok(SimulateFullResponse {
        tx_flow: format!(
            "一次性发送 {} 字节\nHEX: {}",
            packet.len(),
            to_hex(packet)
        ),
        client_mem: render_client_memory(packet, &[]),
        client_struct: render_struct_mapping_details(packet),
        teach_panel: render_teaching_story(packet, "", packet.len() * 8),
        verify,
    })
}

pub fn prepare_bit_mode(form: &ProtocolForm) -> Result<(BuildResponse, String, usize, String), String> {
    let pkt = build_packet(form)?;
    let bits = bytes_to_bit_string(&pkt.packet);
    let total = bits.len();
    let build = BuildResponse {
        packet_hex: to_hex(&pkt.packet),
        bit_desc: render_bit_layout(&pkt),
        server_struct: render_server_protocol_table(form, &pkt),
        packet_len: pkt.packet.len(),
    };
    let tx_flow = format!(
        "已准备按位发送，总位数: {total} ({len} 字节)\n点击「单步发送 1 位」逐步传输。",
        len = pkt.packet.len()
    );
    Ok((build, tx_flow, total, bits))
}

pub fn step_preview(packet_hex: &str, bit_stream: &str) -> Result<StepPreviewResponse, String> {
    let packet = parse_hex(packet_hex)?;
    let bits = bytes_to_bit_string(&packet);
    let sent = bit_stream.len();
    let partial = bits_to_bytes_partial(bit_stream);
    Ok(StepPreviewResponse {
        tx_flow: render_step_flow(bits.len(), sent, bit_stream),
        client_mem: render_client_memory(&partial, bit_stream.as_bytes()),
        teach_panel: render_teaching_story(&packet, bit_stream, bits.len()),
    })
}

pub fn finalize_bits(packet_hex: &str, bit_stream: &str) -> Result<BitFinalizeResponse, String> {
    let packet = parse_hex(packet_hex)?;
    let bytes = bits_to_bytes(bit_stream)?;
    let mut verify = format!("按位重组报文 HEX: {}\n", to_hex(&bytes));
    let client_struct = render_struct_mapping_details(&bytes);
    let teach_panel = render_teaching_story(&packet, bit_stream, bytes.len() * 8);
    match parse_packet(&bytes) {
        Ok(parsed) => {
            verify.push_str(&render_parsed(&parsed));
            let match_hex = to_hex(&bytes) == to_hex(&packet);
            verify.push_str(if match_hex {
                "\n校验: PASS（按位重组 == 编码报文）"
            } else {
                "\n校验: FAIL（按位重组与编码报文不一致）"
            });
        }
        Err(e) => verify.push_str(&format!("解析失败: {e}\n校验: FAIL")),
    }
    Ok(BitFinalizeResponse {
        client_mem: render_client_memory(&bytes, bit_stream.as_bytes()),
        client_struct,
        teach_panel,
        verify,
    })
}

pub fn parse_hex(input: &str) -> Result<Vec<u8>, String> {
    let cleaned = input.replace(',', " ");
    let mut out = Vec::new();
    for token in cleaned.split_whitespace() {
        if token.is_empty() {
            continue;
        }
        let byte = u8::from_str_radix(token, 16)
            .map_err(|_| format!("非法 HEX 字节: {token}"))?;
        out.push(byte);
    }
    if out.is_empty() {
        return Err("报文 HEX 不能为空".to_string());
    }
    Ok(out)
}

fn parse_fc_hex(input: &str) -> Result<Vec<u8>, String> {
    let cleaned = input.replace(',', " ");
    let mut out = Vec::new();
    for token in cleaned.split_whitespace() {
        let byte = u8::from_str_radix(token, 16).map_err(|_| format!("非法 FC 字节: {token}"))?;
        out.push(byte);
    }
    if out.is_empty() {
        return Err("fc 不能为空".to_string());
    }
    for (i, b) in out.iter().enumerate() {
        let has_next = b & 0x01 == 0x01;
        if i < out.len() - 1 && !has_next {
            return Err("fc 非最后字节最低位必须为 1".to_string());
        }
        if i == out.len() - 1 && has_next {
            return Err("fc 最后字节最低位必须为 0".to_string());
        }
    }
    Ok(out)
}

fn pack_data3(lo: u8, hi: u8) -> Result<u8, String> {
    if lo > 7 {
        return Err(format!("data3.lo 值 {lo} 超出 3 位范围"));
    }
    if hi > 31 {
        return Err(format!("data3.hi 值 {hi} 超出 5 位范围"));
    }
    Ok((lo & 0x7) | ((hi & 0x1f) << 3))
}

fn pack_data4(info_c: u16, info_b: u8, info_a: u8) -> Result<u32, String> {
    if info_c > 0x1ff {
        return Err(format!("data4.infoC 值 {info_c} 超出 9 位范围"));
    }
    if info_a > 0x7f {
        return Err(format!("data4.infoA 值 {info_a} 超出 7 位范围"));
    }
    Ok(((info_a as u32) << 17) | ((info_b as u32) << 9) | (info_c as u32))
}

fn data4_to_be_bytes(host: u32) -> [u8; 3] {
    [
        ((host >> 16) & 0xff) as u8,
        ((host >> 8) & 0xff) as u8,
        (host & 0xff) as u8,
    ]
}

fn data4_from_be_bytes(b: [u8; 3]) -> u32 {
    ((b[0] as u32) << 16) | ((b[1] as u32) << 8) | (b[2] as u32)
}

/// 载荷尾部长：data1(2)+data2(4)+data3(1)+data4(3)=10
const TAIL_LEN: usize = 2 + 4 + 1 + 3;

pub fn build_packet(form: &ProtocolForm) -> Result<PacketBuild, String> {
    let fc = parse_fc_hex(&form.fc_hex)?;
    let host_data1 = pack_decl_order_u16(&[
        ("code", 10, form.data1_code as u32),
        ("s3", 1, form.data1_s3 as u32),
        ("s2", 1, form.data1_s2 as u32),
        ("s1", 1, form.data1_s1 as u32),
        ("typeCode", 3, form.data1_type_code as u32),
    ])?;
    let host_data2 = pack_decl_order_u32(&[
        ("feild_1", 4, form.data2_feild_1 as u64),
        ("feild_2", 4, form.data2_feild_2 as u64),
        ("feild_3", 4, form.data2_feild_3 as u64),
        ("feild_4", 4, form.data2_feild_4 as u64),
        ("sysCode", 16, form.data2_sys_code as u64),
    ])?;
    let host_data3 = pack_data3(form.data3_lo, form.data3_hi)?;
    let host_data4 = pack_data4(form.data4_info_c, form.data4_info_b, form.data4_info_a)?;

    // 与历史实现一致：声明序主机值 → swap → to_be_bytes，线上得到声明序的「低字节在前」布局
    let data1 = host_data1.swap_bytes();
    let data2 = host_data2.swap_bytes();
    let data3 = host_data3; // 单字节无端序
    let data4_be = data4_to_be_bytes(host_data4);

    let payload_len = (fc.len() + TAIL_LEN) as u16;
    let mut packet = Vec::with_capacity(payload_len as usize + 3);
    packet.push(form.msg_type);
    packet.extend_from_slice(&payload_len.to_be_bytes());
    packet.extend_from_slice(&fc);
    packet.extend_from_slice(&data1.to_be_bytes());
    packet.extend_from_slice(&data2.to_be_bytes());
    packet.push(data3);
    packet.extend_from_slice(&data4_be);

    Ok(PacketBuild {
        packet,
        host_data1_raw: host_data1,
        host_data2_raw: host_data2,
        host_data3_raw: host_data3,
        host_data4_raw: host_data4,
        data1_raw: data1,
        data2_raw: data2,
        data3_raw: data3,
        data4_be,
    })
}

pub fn parse_packet(packet: &[u8]) -> Result<ParsedPacket, String> {
    if packet.len() < 1 + 2 + TAIL_LEN {
        return Err("报文长度不足".to_string());
    }
    let msg_type = packet[0];
    let len = u16::from_be_bytes([packet[1], packet[2]]);
    let expected = 3 + len as usize;
    if packet.len() != expected {
        return Err(format!("len 不匹配: len={len}, 实际={}", packet.len() - 3));
    }

    let mut idx = 3;
    let mut fc = Vec::new();
    loop {
        if idx >= packet.len() {
            return Err("fc 解析越界".to_string());
        }
        let b = packet[idx];
        idx += 1;
        fc.push(b);
        if b & 0x01 == 0 {
            break;
        }
    }
    if idx + TAIL_LEN != packet.len() {
        return Err("fc 长度与 data1..data4 边界不匹配".to_string());
    }

    let net_data1 = u16::from_be_bytes([packet[idx], packet[idx + 1]]);
    idx += 2;
    let net_data2 = u32::from_be_bytes([
        packet[idx],
        packet[idx + 1],
        packet[idx + 2],
        packet[idx + 3],
    ]);
    idx += 4;
    let data3 = packet[idx];
    idx += 1;
    let data4_be = [packet[idx], packet[idx + 1], packet[idx + 2]];

    let host_data1 = net_data1.swap_bytes();
    let host_data2 = net_data2.swap_bytes();
    let host_data4 = data4_from_be_bytes(data4_be);

    let data1_parts = unpack_decl_order_u16(
        host_data1,
        &[("code", 10), ("s3", 1), ("s2", 1), ("s1", 1), ("typeCode", 3)],
    )?;
    let data2_parts = unpack_decl_order_u32(
        host_data2,
        &[
            ("feild_1", 4),
            ("feild_2", 4),
            ("feild_3", 4),
            ("feild_4", 4),
            ("sysCode", 16),
        ],
    )?;

    Ok(ParsedPacket {
        msg_type,
        len,
        fc,
        data1_type_code: data1_parts[4] as u8,
        data1_s1: data1_parts[3] == 1,
        data1_s2: data1_parts[2] == 1,
        data1_s3: data1_parts[1] == 1,
        data1_code: data1_parts[0] as u16,
        data2_feild_2: data2_parts[1] as u8,
        data2_feild_1: data2_parts[0] as u8,
        data2_feild_4: data2_parts[3] as u8,
        data2_feild_3: data2_parts[2] as u8,
        data2_sys_code: data2_parts[4] as u16,
        data3_lo: data3 & 0x7,
        data3_hi: (data3 >> 3) & 0x1f,
        data4_info_c: (host_data4 & 0x1ff) as u16,
        data4_info_b: ((host_data4 >> 9) & 0xff) as u8,
        data4_info_a: ((host_data4 >> 17) & 0x7f) as u8,
    })
}

fn render_parsed(p: &ParsedPacket) -> String {
    format!(
        "type=0x{:02X}, len={}, fc=[{}], \
         data1(typeCode={}, s1={}, s2={}, s3={}, code={}), \
         data2(feild_2={}, feild_1={}, feild_4={}, feild_3={}, sysCode=0x{:04X}), \
         data3(lo={}, hi={}), \
         data4(infoA=0x{:02X}, infoB=0x{:02X}, infoC=0x{:03X})",
        p.msg_type,
        p.len,
        p.fc.iter()
            .map(|b| format!("{:02X}", b))
            .collect::<Vec<_>>()
            .join(" "),
        p.data1_type_code,
        p.data1_s1 as u8,
        p.data1_s2 as u8,
        p.data1_s3 as u8,
        p.data1_code,
        p.data2_feild_2,
        p.data2_feild_1,
        p.data2_feild_4,
        p.data2_feild_3,
        p.data2_sys_code,
        p.data3_lo,
        p.data3_hi,
        p.data4_info_a,
        p.data4_info_b,
        p.data4_info_c
    )
}

fn render_bit_layout(pkt: &PacketBuild) -> String {
    format!(
        "按声明顺序(主机序) data1=code,s3,s2,s1,typeCode => {:016b}\n\
         按声明顺序(主机序) data2=feild_1,feild_2,feild_3,feild_4,sysCode => {:032b}\n\
         按声明顺序(主机序) data3=lo(3),hi(5) => {:08b}\n\
         按声明顺序(主机序) data4=infoC(9),infoB(8),infoA(7) => {:024b}\n\
         转网络序后 data1(be)={:016b}, data2(be)={:032b}, data3={:02X}, data4(be)={:02X} {:02X} {:02X}\n\
         语义位段: data1 [15..13]=typeCode [12]=s1 [11]=s2 [10]=s3 [9..0]=code\n\
         语义位段: data2 [31..28]=feild_2 [27..24]=feild_1 [23..20]=feild_4 [19..16]=feild_3 [15..0]=sysCode\n\
         语义位段: data3 [7..3]=hi [2..0]=lo\n\
         语义位段: data4 [23..17]=infoA [16..9]=infoB [8..0]=infoC",
        pkt.host_data1_raw,
        pkt.host_data2_raw,
        pkt.host_data3_raw,
        pkt.host_data4_raw,
        pkt.data1_raw,
        pkt.data2_raw,
        pkt.data3_raw,
        pkt.data4_be[0],
        pkt.data4_be[1],
        pkt.data4_be[2]
    )
}

fn pack_decl_order_u16(fields: &[(&str, u8, u32)]) -> Result<u16, String> {
    let mut offset = 0u8;
    let mut raw = 0u16;
    for (name, bits, value) in fields {
        if *bits == 0 || *bits > 16 {
            return Err(format!("字段 {name} 位宽非法: {bits}"));
        }
        if offset + *bits > 16 {
            return Err(format!("u16 超出位宽, 字段: {name}"));
        }
        let max = (1u32 << *bits) - 1;
        if *value > max {
            return Err(format!("字段 {name} 值 {value} 超出 {bits} 位范围"));
        }
        raw |= ((*value as u16) & ((1u16 << *bits) - 1)) << offset;
        offset += *bits;
    }
    if offset != 16 {
        return Err(format!("data1 总位宽应为16, 当前为{offset}"));
    }
    Ok(raw)
}

fn pack_decl_order_u32(fields: &[(&str, u8, u64)]) -> Result<u32, String> {
    let mut offset = 0u8;
    let mut raw = 0u32;
    for (name, bits, value) in fields {
        if *bits == 0 || *bits > 32 {
            return Err(format!("字段 {name} 位宽非法: {bits}"));
        }
        if offset + *bits > 32 {
            return Err(format!("u32 超出位宽, 字段: {name}"));
        }
        let max = (1u64 << *bits) - 1;
        if *value > max {
            return Err(format!("字段 {name} 值 {value} 超出 {bits} 位范围"));
        }
        raw |= ((*value as u32) & ((1u32 << *bits) - 1)) << offset;
        offset += *bits;
    }
    if offset != 32 {
        return Err(format!("data2 总位宽应为32, 当前为{offset}"));
    }
    Ok(raw)
}

fn unpack_decl_order_u16(raw: u16, fields: &[(&str, u8)]) -> Result<Vec<u32>, String> {
    let mut out = Vec::with_capacity(fields.len());
    let mut offset = 0u8;
    for (name, bits) in fields {
        if *bits == 0 || offset + *bits > 16 {
            return Err(format!("u16 解包配置非法: {name}"));
        }
        let mask = (1u16 << *bits) - 1;
        out.push(((raw >> offset) & mask) as u32);
        offset += *bits;
    }
    Ok(out)
}

fn unpack_decl_order_u32(raw: u32, fields: &[(&str, u8)]) -> Result<Vec<u64>, String> {
    let mut out = Vec::with_capacity(fields.len());
    let mut offset = 0u8;
    for (name, bits) in fields {
        if *bits == 0 || offset + *bits > 32 {
            return Err(format!("u32 解包配置非法: {name}"));
        }
        let mask = (1u32 << *bits) - 1;
        out.push(((raw >> offset) & mask) as u64);
        offset += *bits;
    }
    Ok(out)
}

fn bits_to_bytes(bits: &str) -> Result<Vec<u8>, String> {
    if bits.len() % 8 != 0 {
        return Err("位流长度不是 8 的倍数".to_string());
    }
    let mut out = Vec::with_capacity(bits.len() / 8);
    for chunk in bits.as_bytes().chunks(8) {
        let mut v = 0u8;
        for &b in chunk {
            v <<= 1;
            if b == b'1' {
                v |= 1;
            } else if b != b'0' {
                return Err("位流中存在非法字符".to_string());
            }
        }
        out.push(v);
    }
    Ok(out)
}

pub fn to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

fn bytes_to_bit_string(bytes: &[u8]) -> String {
    let mut out = String::with_capacity(bytes.len() * 8);
    for b in bytes {
        for i in (0..8).rev() {
            out.push(if (b >> i) & 1 == 1 { '1' } else { '0' });
        }
    }
    out
}

fn bits_to_bytes_partial(bits: &str) -> Vec<u8> {
    let full = bits.len() / 8;
    let mut out = Vec::with_capacity(full);
    for chunk in bits.as_bytes().chunks(8).take(full) {
        let mut v = 0u8;
        for &b in chunk {
            v <<= 1;
            if b == b'1' {
                v |= 1;
            }
        }
        out.push(v);
    }
    out
}

fn render_step_flow(total_bits: usize, sent_bits: usize, stream: &str) -> String {
    let preview = if stream.len() > 128 {
        format!("...{}", &stream[stream.len() - 128..])
    } else {
        stream.to_string()
    };
    format!(
        "传输进度: {sent_bits}/{total_bits} bit\n最新位流(尾部预览):\n{preview}\n\n说明:\n- 每次点击仅发送 1 bit\n- 8 bit 自动组装成 1 字节进入客户端内存缓冲\n- 全部发送后点击「完成按位帧并解析」"
    )
}

fn render_client_memory(bytes: &[u8], raw_bits: &[u8]) -> String {
    let mut s = String::new();
    let bit_text = String::from_utf8_lossy(raw_bits);
    let _ = writeln!(s, "客户端位缓冲区(bit): {bit_text}");
    let _ = writeln!(s, "客户端字节缓冲区(hex): {}", to_hex(bytes));
    for (i, b) in bytes.iter().enumerate() {
        let _ = writeln!(s, "byte[{i:02}] = 0x{b:02X} = {:08b}", b);
    }
    s
}

fn render_server_protocol_table(form: &ProtocolForm, pkt: &PacketBuild) -> String {
    let fc = parse_fc_hex(&form.fc_hex).unwrap_or_default();
    let mut s = String::new();
    let _ = writeln!(s, "协议结构(网络传输顺序):");
    let _ = writeln!(s, "1) type  : 1B  = 0x{:02X}", form.msg_type);
    let _ = writeln!(
        s,
        "2) len   : 2B  = {} (0x{:04X})  [= fc + data1(2)+data2(4)+data3(1)+data4(3)]",
        fc.len() + TAIL_LEN,
        (fc.len() + TAIL_LEN) as u16
    );
    let _ = writeln!(s, "3) fc    : {}B = {}", fc.len(), to_hex(&fc));
    let _ = writeln!(s, "4) data1 : 2B  host=0x{:04X}  net=0x{:04X}", pkt.host_data1_raw, pkt.data1_raw);
    let _ = writeln!(
        s,
        "   - code(10b)={} s3={} s2={} s1={} typeCode={}",
        form.data1_code,
        form.data1_s3 as u8,
        form.data1_s2 as u8,
        form.data1_s1 as u8,
        form.data1_type_code
    );
    let _ = writeln!(s, "5) data2 : 4B  host=0x{:08X}  net=0x{:08X}", pkt.host_data2_raw, pkt.data2_raw);
    let _ = writeln!(
        s,
        "   - feild_1={} feild_2={} feild_3={} feild_4={} sysCode=0x{:04X}",
        form.data2_feild_1, form.data2_feild_2, form.data2_feild_3, form.data2_feild_4, form.data2_sys_code
    );
    let _ = writeln!(s, "6) data3 : 1B  = 0x{:02X}", pkt.data3_raw);
    let _ = writeln!(
        s,
        "   - lo[2:0]={} hi[7:3]={}",
        form.data3_lo & 7,
        form.data3_hi & 0x1f
    );
    let _ = writeln!(
        s,
        "7) data4 : 3B  host=0x{:06X}  net={:02X} {:02X} {:02X}",
        pkt.host_data4_raw, pkt.data4_be[0], pkt.data4_be[1], pkt.data4_be[2]
    );
    let _ = writeln!(
        s,
        "   - infoA[23:17]=0x{:02X} infoB[16:9]=0x{:02X} infoC[8:0]=0x{:03X}",
        form.data4_info_a & 0x7f,
        form.data4_info_b,
        form.data4_info_c & 0x1ff
    );
    let _ = writeln!(s, "\n完整报文(hex): {}", to_hex(&pkt.packet));
    s
}

fn render_struct_mapping_details(packet: &[u8]) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "Step A: 收到网络报文字节流");
    let _ = writeln!(s, "HEX: {}", to_hex(packet));
    if packet.len() < 3 + TAIL_LEN {
        let _ = writeln!(s, "报文太短，无法映射。");
        return s;
    }
    let _ = writeln!(s, "\nStep B: memcpy 到 packed 内存块（按字节连续）");
    for (i, b) in packet.iter().enumerate() {
        let _ = writeln!(s, "mem[{i:02}] = 0x{b:02X}");
    }
    let _ = writeln!(s, "\nStep C: 切片 data1/data2/data3/data4 并 ntoh / fromNet");
    match parse_packet(packet) {
        Ok(p) => {
            let _ = writeln!(
                s,
                "data1 => typeCode={}, s1={}, s2={}, s3={}, code={}",
                p.data1_type_code, p.data1_s1 as u8, p.data1_s2 as u8, p.data1_s3 as u8, p.data1_code
            );
            let _ = writeln!(
                s,
                "data2 => feild_2={}, feild_1={}, feild_4={}, feild_3={}, sysCode=0x{:04X}",
                p.data2_feild_2, p.data2_feild_1, p.data2_feild_4, p.data2_feild_3, p.data2_sys_code
            );
            let _ = writeln!(s, "data3 => lo={}, hi={} (单字节，无需端序转换)", p.data3_lo, p.data3_hi);
            let _ = writeln!(
                s,
                "data4 => infoA=0x{:02X}, infoB=0x{:02X}, infoC=0x{:03X}",
                p.data4_info_a, p.data4_info_b, p.data4_info_c
            );
            let _ = writeln!(s, "\nStep D: 按结构体声明顺序解包位域 (低位->高位)");
            let _ = writeln!(s, "data1: code -> s3 -> s2 -> s1 -> typeCode");
            let _ = writeln!(s, "data2: feild_1 -> feild_2 -> feild_3 -> feild_4 -> sysCode");
            let _ = writeln!(s, "data3: lo(3) -> hi(5)");
            let _ = writeln!(s, "data4: infoC(9) -> infoB(8) -> infoA(7)");
        }
        Err(e) => {
            let _ = writeln!(s, "解析失败: {e}");
        }
    }
    s
}

fn render_teaching_story(packet: &[u8], received_bits: &str, total_bits: usize) -> String {
    let mut s = String::new();
    let recv = received_bits.len();
    let _ = writeln!(s, "【Step 1: 网络逐位接收】");
    let _ = writeln!(s, "已接收位数: {recv}/{total_bits}");
    if recv == 0 {
        let _ = writeln!(s, "等待点击「单步发送 1 位」。");
    } else {
        let preview = if received_bits.len() > 96 {
            format!("...{}", &received_bits[received_bits.len() - 96..])
        } else {
            received_bits.to_string()
        };
        let _ = writeln!(s, "位流尾部: {preview}");
    }

    let partial_bytes = bits_to_bytes_partial(received_bits);
    let _ = writeln!(s, "\n【Step 2: 每8位组1字节】");
    let _ = writeln!(s, "当前已组装字节: {} byte", partial_bytes.len());
    let _ = writeln!(s, "HEX: {}", to_hex(&partial_bytes));

    if partial_bytes.len() < 3 {
        let _ = writeln!(s, "\n报文头(type+len)尚未接收完整。");
        return s;
    }

    let msg_type = partial_bytes[0];
    let len = u16::from_be_bytes([partial_bytes[1], partial_bytes[2]]);
    let _ = writeln!(s, "\n【Step 3: 报文头语义】");
    let _ = writeln!(s, "type=0x{msg_type:02X}, len={len}");

    if partial_bytes.len() < packet.len() {
        let _ = writeln!(s, "\n报文未收全，尾部字段还不能做完整 ntoh+位域映射。");
        return s;
    }

    let _ = writeln!(s, "\n【Step 4: data1..data4 网络序 -> 主机序】");
    if let Ok(parsed) = parse_packet(packet) {
        let full_hex = to_hex(packet);
        let _ = writeln!(s, "完整报文: {full_hex}");
        let _ = writeln!(
            s,
            "data1: code={} | s3={} | s2={} | s1={} | typeCode={}",
            parsed.data1_code,
            parsed.data1_s3 as u8,
            parsed.data1_s2 as u8,
            parsed.data1_s1 as u8,
            parsed.data1_type_code
        );
        let _ = writeln!(
            s,
            "data2: feild_1={} | feild_2={} | feild_3={} | feild_4={} | sysCode=0x{:04X}",
            parsed.data2_feild_1,
            parsed.data2_feild_2,
            parsed.data2_feild_3,
            parsed.data2_feild_4,
            parsed.data2_sys_code
        );
        let _ = writeln!(s, "data3: lo={} | hi={}", parsed.data3_lo, parsed.data3_hi);
        let _ = writeln!(
            s,
            "data4: infoA=0x{:02X} | infoB=0x{:02X} | infoC=0x{:03X}",
            parsed.data4_info_a, parsed.data4_info_b, parsed.data4_info_c
        );
        let _ = writeln!(s, "\n【Step 5: 位域声明顺序】");
        let _ = writeln!(s, "- 谁先声明谁占低位(bit0 起)");
        let _ = writeln!(s, "- 多字节字段先 ntoh/fromNet，再读位域成员");
    }
    s
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn round_trip_default_form() {
        let form = ProtocolForm::default();
        let pkt = build_packet(&form).unwrap();
        // 图解示例：len=13, 总长 16
        assert_eq!(pkt.packet.len(), 16);
        assert_eq!(&pkt.packet[0..3], &[0x11, 0x00, 0x0D]);
        assert_eq!(&pkt.packet[3..6], &[0x51, 0x73, 0xB8]);
        assert_eq!(&pkt.packet[6..8], &[0x55, 0xB5]);
        assert_eq!(&pkt.packet[8..12], &[0x93, 0x65, 0x34, 0x12]);
        assert_eq!(pkt.packet[12], 0xAD);
        assert_eq!(&pkt.packet[13..16], &[0xAB, 0x55, 0x55]);

        let parsed = parse_packet(&pkt.packet).unwrap();
        assert_eq!(parsed.msg_type, form.msg_type);
        assert_eq!(parsed.data1_code, form.data1_code);
        assert_eq!(parsed.data1_type_code, form.data1_type_code);
        assert_eq!(parsed.data2_sys_code, form.data2_sys_code);
        assert_eq!(parsed.data3_lo, form.data3_lo);
        assert_eq!(parsed.data3_hi, form.data3_hi);
        assert_eq!(parsed.data4_info_a, form.data4_info_a);
        assert_eq!(parsed.data4_info_b, form.data4_info_b);
        assert_eq!(parsed.data4_info_c, form.data4_info_c);
    }

    #[test]
    fn random_round_trip() {
        for _ in 0..20 {
            let form = random_form();
            let pkt = build_packet(&form).unwrap();
            let parsed = parse_packet(&pkt.packet).unwrap();
            assert_eq!(parsed.data1_code, form.data1_code);
            assert_eq!(parsed.data3_lo, form.data3_lo & 7);
            assert_eq!(parsed.data3_hi, form.data3_hi & 0x1f);
            assert_eq!(parsed.data4_info_c, form.data4_info_c & 0x1ff);
            assert_eq!(parsed.data4_info_a, form.data4_info_a & 0x7f);
        }
    }
}
