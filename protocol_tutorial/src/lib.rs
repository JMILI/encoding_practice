use rand::Rng;
use serde::{Deserialize, Serialize};
use std::fmt::Write as _;

#[derive(Clone, Debug, Serialize, Deserialize)]
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
        }
    }
}

#[derive(Clone, Debug, Serialize)]
pub struct PacketBuild {
    pub packet: Vec<u8>,
    pub host_data1_raw: u16,
    pub host_data2_raw: u32,
    pub data1_raw: u16,
    pub data2_raw: u32,
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
    let mut verify = format!(
        "一次发送报文 HEX: {}\n",
        to_hex(packet)
    );
    match parse_packet(packet) {
        Ok(p) => {
            verify.push_str(&render_parsed(&p));
            verify.push_str("\n校验: PASS");
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
            verify.push_str("\n校验: PASS");
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
    let data1 = host_data1.swap_bytes();
    let data2 = host_data2.swap_bytes();

    let payload_len = (fc.len() + 2 + 4) as u16;
    let mut packet = Vec::with_capacity(payload_len as usize + 3);
    packet.push(form.msg_type);
    packet.extend_from_slice(&payload_len.to_be_bytes());
    packet.extend_from_slice(&fc);
    packet.extend_from_slice(&data1.to_be_bytes());
    packet.extend_from_slice(&data2.to_be_bytes());

    Ok(PacketBuild {
        packet,
        host_data1_raw: host_data1,
        host_data2_raw: host_data2,
        data1_raw: data1,
        data2_raw: data2,
    })
}

pub fn parse_packet(packet: &[u8]) -> Result<ParsedPacket, String> {
    if packet.len() < 1 + 2 + 2 + 4 {
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
    if idx + 2 + 4 != packet.len() {
        return Err("fc 长度与 data1/data2 边界不匹配".to_string());
    }

    let net_data1 = u16::from_be_bytes([packet[idx], packet[idx + 1]]);
    idx += 2;
    let net_data2 = u32::from_be_bytes([packet[idx], packet[idx + 1], packet[idx + 2], packet[idx + 3]]);
    let host_data1 = net_data1.swap_bytes();
    let host_data2 = net_data2.swap_bytes();

    let data1_parts = unpack_decl_order_u16(
        host_data1,
        &[("code", 10), ("s3", 1), ("s2", 1), ("s1", 1), ("typeCode", 3)],
    )?;
    let data2_parts = unpack_decl_order_u32(
        host_data2,
        &[("feild_1", 4), ("feild_2", 4), ("feild_3", 4), ("feild_4", 4), ("sysCode", 16)],
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
    })
}

fn render_parsed(p: &ParsedPacket) -> String {
    format!(
        "type=0x{:#04X}, len={}, fc=[{}], data1(typeCode={}, s1={}, s2={}, s3={}, code={}), data2(feild_2={}, feild_1={}, feild_4={}, feild_3={}, sysCode=0x{:04X})",
        p.msg_type,
        p.len,
        p.fc.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "),
        p.data1_type_code,
        p.data1_s1 as u8,
        p.data1_s2 as u8,
        p.data1_s3 as u8,
        p.data1_code,
        p.data2_feild_2,
        p.data2_feild_1,
        p.data2_feild_4,
        p.data2_feild_3,
        p.data2_sys_code
    )
}

fn render_bit_layout(pkt: &PacketBuild) -> String {
    format!(
        "按声明顺序(主机序) data1=code,s3,s2,s1,typeCode => {:016b}\n\
         按声明顺序(主机序) data2=feild_1,feild_2,feild_3,feild_4,sysCode => {:032b}\n\
         转网络序后 data1(be)={:016b}, data2(be)={:032b}\n\
         语义位段(协议定义): data1 [15..13]=typeCode [12]=s1 [11]=s2 [10]=s3 [9..0]=code\n\
         语义位段(协议定义): data2 [31..28]=feild_2 [27..24]=feild_1 [23..20]=feild_4 [19..16]=feild_3 [15..0]=sysCode",
        pkt.host_data1_raw, pkt.host_data2_raw, pkt.data1_raw, pkt.data2_raw
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
    let _ = writeln!(s, "2) len   : 2B  = {} (0x{:04X})", fc.len() + 6, (fc.len() + 6) as u16);
    let _ = writeln!(s, "3) fc    : {}B = {}", fc.len(), to_hex(&fc));
    let _ = writeln!(s, "4) data1 : 2B  = 0x{:04X}", pkt.data1_raw);
    let _ = writeln!(
        s,
        "   - code(10b)={} s3={} s2={} s1={} typeCode={}",
        form.data1_code,
        form.data1_s3 as u8,
        form.data1_s2 as u8,
        form.data1_s1 as u8,
        form.data1_type_code
    );
    let _ = writeln!(s, "5) data2 : 4B  = 0x{:08X}", pkt.data2_raw);
    let _ = writeln!(
        s,
        "   - feild_1={} feild_2={} feild_3={} feild_4={} sysCode=0x{:04X}",
        form.data2_feild_1, form.data2_feild_2, form.data2_feild_3, form.data2_feild_4, form.data2_sys_code
    );
    let _ = writeln!(s, "\n完整报文(hex): {}", to_hex(&pkt.packet));
    s
}

fn render_struct_mapping_details(packet: &[u8]) -> String {
    let mut s = String::new();
    let _ = writeln!(s, "Step A: 收到网络报文字节流");
    let _ = writeln!(s, "HEX: {}", to_hex(packet));
    if packet.len() < 9 {
        let _ = writeln!(s, "报文太短，无法映射。");
        return s;
    }
    let _ = writeln!(s, "\nStep B: memcpy 到 packed 内存块（按字节连续）");
    for (i, b) in packet.iter().enumerate() {
        let _ = writeln!(s, "mem[{i:02}] = 0x{b:02X}");
    }
    let _ = writeln!(s, "\nStep C: 从 mem 中切片出 data1/data2 并 ntoh");
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
            let _ = writeln!(s, "\nStep D: 按结构体声明顺序解包位域 (低位->高位)");
            let _ = writeln!(s, "data1: code -> s3 -> s2 -> s1 -> typeCode");
            let _ = writeln!(s, "data2: feild_1 -> feild_2 -> feild_3 -> feild_4 -> sysCode");
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
        let _ = writeln!(s, "\n报文未收全，data1/data2 还不能做完整 ntoh+位域映射。");
        return s;
    }

    let _ = writeln!(s, "\n【Step 4: data1/data2 网络序 -> 主机序】");
    if let Ok(parsed) = parse_packet(packet) {
        let full_hex = to_hex(packet);
        let _ = writeln!(s, "完整报文: {full_hex}");
        let _ = writeln!(s, "data1 字段映射结果:");
        let _ = writeln!(
            s,
            "  code={} | s3={} | s2={} | s1={} | typeCode={}",
            parsed.data1_code,
            parsed.data1_s3 as u8,
            parsed.data1_s2 as u8,
            parsed.data1_s1 as u8,
            parsed.data1_type_code
        );
        let _ = writeln!(s, "data2 字段映射结果:");
        let _ = writeln!(
            s,
            "  feild_1={} | feild_2={} | feild_3={} | feild_4={} | sysCode=0x{:04X}",
            parsed.data2_feild_1,
            parsed.data2_feild_2,
            parsed.data2_feild_3,
            parsed.data2_feild_4,
            parsed.data2_sys_code
        );
        let _ = writeln!(s, "\n【Step 5: 为什么结构体字段顺序看起来「反着写」】");
        let _ = writeln!(s, "- 协议文档位号是「语义编号」(如 feild_2=32..29)");
        let _ = writeln!(s, "- 位域声明顺序是「主机内存落位顺序」(低位字段通常先声明)");
        let _ = writeln!(s, "- 再经过 ntoh/hton 字节序转换，最终与协议语义位号对齐");
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
        let parsed = parse_packet(&pkt.packet).unwrap();
        assert_eq!(parsed.msg_type, form.msg_type);
        assert_eq!(parsed.data1_code, form.data1_code);
        assert_eq!(parsed.data2_sys_code, form.data2_sys_code);
    }
}
