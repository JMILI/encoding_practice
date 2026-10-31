use dioxus::prelude::*;
use rand::Rng;
use std::fmt::Write as _;
use std::net::UdpSocket;
use std::sync::mpsc::{self, Receiver, Sender};
use std::thread;
use std::time::Duration;

const SERVER_ADDR: &str = "127.0.0.1:44001";
const CLIENT_ADDR: &str = "127.0.0.1:44002";

fn main() {
    dioxus::launch(App);
}

#[derive(Clone, Debug)]
struct ProtocolForm {
    msg_type: u8,
    fc_hex: String,
    data1_type_code: u8,
    data1_s1: bool,
    data1_s2: bool,
    data1_s3: bool,
    data1_code: u16,
    data2_dq: u8,
    data2_fdq: u8,
    data2_jj: u8,
    data2_bz: u8,
    data2_sys_code: u16,
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
            data2_dq: 0b1001,
            data2_fdq: 0b0011,
            data2_jj: 0b0110,
            data2_bz: 0b0101,
            data2_sys_code: 0x1234,
        }
    }
}

#[derive(Clone, Debug)]
struct PacketBuild {
    packet: Vec<u8>,
    host_data1_raw: u16,
    host_data2_raw: u32,
    data1_raw: u16,
    data2_raw: u32,
}

#[derive(Clone, Debug)]
struct ParsedPacket {
    msg_type: u8,
    len: u16,
    fc: Vec<u8>,
    data1_type_code: u8,
    data1_s1: bool,
    data1_s2: bool,
    data1_s3: bool,
    data1_code: u16,
    data2_dq: u8,
    data2_fdq: u8,
    data2_jj: u8,
    data2_bz: u8,
    data2_sys_code: u16,
}

#[derive(Clone, Debug)]
enum ClientEvent {
    Packet(Vec<u8>),
    BitFrame(Vec<u8>),
}

struct UdpLab {
    sender: UdpSocket,
    events_rx: Receiver<ClientEvent>,
}

impl UdpLab {
    fn new() -> Result<Self, String> {
        let sender = UdpSocket::bind(SERVER_ADDR).map_err(|e| format!("服务端绑定失败: {e}"))?;
        sender
            .set_nonblocking(false)
            .map_err(|e| format!("服务端套接字配置失败: {e}"))?;

        let (tx, rx): (Sender<ClientEvent>, Receiver<ClientEvent>) = mpsc::channel();
        thread::spawn(move || {
            let client = match UdpSocket::bind(CLIENT_ADDR) {
                Ok(s) => s,
                Err(_) => return,
            };
            let _ = client.set_read_timeout(Some(Duration::from_millis(200)));
            let mut buf = [0u8; 2048];
            let mut bit_buffer = String::new();
            loop {
                match client.recv_from(&mut buf) {
                    Ok((n, _)) => {
                        let payload = &buf[..n];
                        if payload == b"BIT_END" {
                            if let Ok(bytes) = bits_to_bytes(&bit_buffer) {
                                let _ = tx.send(ClientEvent::BitFrame(bytes));
                            }
                            bit_buffer.clear();
                            continue;
                        }
                        if payload.len() == 1 && (payload[0] == b'0' || payload[0] == b'1') {
                            bit_buffer.push(payload[0] as char);
                            continue;
                        }
                        let _ = tx.send(ClientEvent::Packet(payload.to_vec()));
                    }
                    Err(_) => {}
                }
            }
        });

        Ok(Self {
            sender,
            events_rx: rx,
        })
    }

    fn send_packet(&self, bytes: &[u8]) -> Result<(), String> {
        self.sender
            .send_to(bytes, CLIENT_ADDR)
            .map(|_| ())
            .map_err(|e| format!("UDP 一次发送失败: {e}"))
    }

    fn drain_events(&self) -> Vec<ClientEvent> {
        let mut out = Vec::new();
        while let Ok(event) = self.events_rx.try_recv() {
            out.push(event);
        }
        out
    }
}

#[component]
fn App() -> Element {
    let mut form = use_signal(ProtocolForm::default);
    let logs = use_signal(Vec::<String>::new);
    let packet_hex = use_signal(String::new);
    let bit_desc = use_signal(String::new);
    let verify = use_signal(String::new);
    let server_struct = use_signal(String::new);
    let tx_flow = use_signal(String::new);
    let client_mem = use_signal(String::new);
    let client_struct = use_signal(String::new);
    let teach_panel = use_signal(String::new);
    let tx_bits = use_signal(Vec::<char>::new);
    let tx_cursor = use_signal(|| 0usize);
    let client_bit_stream = use_signal(String::new);
    let built_packet = use_signal(|| Option::<PacketBuild>::None);
    let lab = use_signal(|| UdpLab::new().ok());

    let randomize = move |_| {
        let mut rng = rand::thread_rng();
        form.set(ProtocolForm {
            msg_type: rng.gen(),
            fc_hex: "81 83 24".to_string(),
            data1_type_code: rng.gen_range(0..=7),
            data1_s1: rng.gen_bool(0.5),
            data1_s2: rng.gen_bool(0.5),
            data1_s3: rng.gen_bool(0.5),
            data1_code: rng.gen_range(0..=1023),
            data2_dq: rng.gen_range(0..=15),
            data2_fdq: rng.gen_range(0..=15),
            data2_jj: rng.gen_range(0..=15),
            data2_bz: rng.gen_range(0..=15),
            data2_sys_code: rng.gen(),
        });
    };

    let send_full = {
        let lab = lab.clone();
        let mut logs = logs.clone();
        let mut verify = verify.clone();
        let mut client_mem = client_mem.clone();
        let mut client_struct = client_struct.clone();
        let mut teach_panel = teach_panel.clone();
        let form = form.clone();
        let mut packet_hex = packet_hex.clone();
        let mut bit_desc = bit_desc.clone();
        let mut server_struct = server_struct.clone();
        let mut tx_flow = tx_flow.clone();
        let mut built_packet = built_packet.clone();
        move |_| {
            if let Some(pkt) = build_and_refresh(&form, &mut logs, &mut packet_hex, &mut bit_desc) {
                server_struct.set(render_server_protocol_table(&form.read(), &pkt));
                tx_flow.set(format!("一次性发送 {} 字节\nHEX: {}", pkt.packet.len(), to_hex(&pkt.packet)));
                built_packet.set(Some(pkt.clone()));
                if let Some(net) = &*lab.read() {
                    match net.send_packet(&pkt.packet) {
                        Ok(_) => {
                            thread::sleep(Duration::from_millis(80));
                            let merged = consume_client_events(net.drain_events(), &mut logs);
                            verify.set(merged);
                            client_mem.set(render_client_memory(&pkt.packet, &[]));
                            client_struct.set(render_struct_mapping_details(&pkt.packet));
                            teach_panel.set(render_teaching_story(&pkt.packet, "", pkt.packet.len() * 8));
                        }
                        Err(e) => logs.push(e),
                    }
                } else {
                    logs.push("网络初始化失败，无法发送".to_string());
                }
            }
        }
    };

    let send_bits = {
        let lab = lab.clone();
        let mut logs = logs.clone();
        let form = form.clone();
        let mut packet_hex = packet_hex.clone();
        let mut bit_desc = bit_desc.clone();
        let mut server_struct = server_struct.clone();
        let mut tx_bits = tx_bits.clone();
        let mut tx_cursor = tx_cursor.clone();
        let mut client_bit_stream = client_bit_stream.clone();
        let mut tx_flow = tx_flow.clone();
        let mut client_mem = client_mem.clone();
        let mut client_struct = client_struct.clone();
        let mut teach_panel = teach_panel.clone();
        let mut verify = verify.clone();
        let mut built_packet = built_packet.clone();
        move |_| {
            if let Some(pkt) = build_and_refresh(&form, &mut logs, &mut packet_hex, &mut bit_desc) {
                server_struct.set(render_server_protocol_table(&form.read(), &pkt));
                let bits = bytes_to_bit_chars(&pkt.packet);
                tx_bits.set(bits.clone());
                tx_cursor.set(0);
                client_bit_stream.set(String::new());
                tx_flow.set(format!(
                    "已准备按位发送，总位数: {} ({} 字节)\n点击“单步发送 1 位”逐步传输。",
                    bits.len(),
                    pkt.packet.len()
                ));
                client_mem.set("客户端内存区为空，等待位流组装成完整字节。".to_string());
                client_struct.set("尚未完成完整报文，结构体无法映射。".to_string());
                teach_panel.set(render_teaching_story(&pkt.packet, "", bits.len()));
                verify.set(String::new());
                built_packet.set(Some(pkt));
                let _ = &lab;
            }
        }
    };

    let step_send_one_bit = {
        let lab = lab.clone();
        let mut logs = logs.clone();
        let mut tx_cursor = tx_cursor.clone();
        let tx_bits = tx_bits.clone();
        let built_packet = built_packet.clone();
        let mut client_bit_stream = client_bit_stream.clone();
        let mut tx_flow = tx_flow.clone();
        let mut client_mem = client_mem.clone();
        let mut teach_panel = teach_panel.clone();
        move |_| {
            let cursor = *tx_cursor.read();
            let bits = tx_bits.read();
            if cursor >= bits.len() {
                logs.push("所有位已发送，请点击“完成按位帧并解析”。".to_string());
                return;
            }
            let bit = bits[cursor];
            if let Some(net) = &*lab.read() {
                let payload = [bit as u8];
                if let Err(e) = net.sender.send_to(&payload, CLIENT_ADDR) {
                    logs.push(format!("发送单个位失败: {e}"));
                    return;
                }
            }
            client_bit_stream.write().push(bit);
            tx_cursor.set(cursor + 1);
            let bit_stream = client_bit_stream.read().clone();
            let partial = bits_to_bytes_partial(&bit_stream);
            client_mem.set(render_client_memory(&partial, bit_stream.as_bytes()));
            tx_flow.set(render_step_flow(bits.len(), cursor + 1, &bit_stream));
            if let Some(pkt) = &*built_packet.read() {
                teach_panel.set(render_teaching_story(&pkt.packet, &bit_stream, bits.len()));
            }
        }
    };

    let finalize_bit_frame = {
        let lab = lab.clone();
        let mut logs = logs.clone();
        let client_bit_stream = client_bit_stream.clone();
        let mut verify = verify.clone();
        let mut client_mem = client_mem.clone();
        let mut client_struct = client_struct.clone();
        let mut teach_panel = teach_panel.clone();
        move |_| {
            if let Some(net) = &*lab.read() {
                let _ = net.sender.send_to(b"BIT_END", CLIENT_ADDR);
            }
            let stream = client_bit_stream.read().clone();
            match bits_to_bytes(&stream) {
                Ok(bytes) => {
                    logs.push(format!("按位传输完成，客户端重组 {} 字节", bytes.len()));
                    client_mem.set(render_client_memory(&bytes, stream.as_bytes()));
                    let mut verify_text = format!("按位重组报文 HEX: {}\n", to_hex(&bytes));
                    client_struct.set(render_struct_mapping_details(&bytes));
                    teach_panel.set(render_teaching_story(&bytes, &stream, bytes.len() * 8));
                    match parse_packet(&bytes) {
                        Ok(parsed) => verify_text.push_str(&format!("{}\n校验: PASS", render_parsed(&parsed))),
                        Err(e) => verify_text.push_str(&format!("解析失败: {e}\n校验: FAIL")),
                    }
                    verify.set(verify_text);
                }
                Err(e) => {
                    logs.push(format!("按位重组失败: {e}"));
                    verify.set(format!("按位重组失败: {e}"));
                    client_struct.set("无法映射结构体（位流长度或内容不合法）".to_string());
                }
            }
        }
    };

    let poll_client = {
        let lab = lab.clone();
        let mut logs = logs.clone();
        let mut verify = verify.clone();
        move |_| {
            if let Some(net) = &*lab.read() {
                let merged = consume_client_events(net.drain_events(), &mut logs);
                if !merged.is_empty() {
                    verify.set(merged);
                }
            }
        }
    };

    rsx! {
        style { {APP_STYLE} }
        div { class: "page",
            h1 { "Rust + Dioxus 协议位域教程实验台" }
            p { class: "subtitle", "支持大端编码、UDP 单播一次发送与按位发送、客户端小端环境解析校验" }

            div { class: "panel",
                h2 { "协议定义（服务端）" }
                div { class: "grid",
                    label { "type (1B):" }
                    input {
                        value: "{form.read().msg_type}",
                        oninput: move |e| if let Ok(v) = e.value().parse::<u8>() { form.write().msg_type = v; }
                    }
                    label { "fc (hex, 可变长):" }
                    input {
                        value: "{form.read().fc_hex}",
                        oninput: move |e| form.write().fc_hex = e.value().to_string()
                    }
                    label { "data1.typeCode(3b):" }
                    input {
                        value: "{form.read().data1_type_code}",
                        oninput: move |e| if let Ok(v) = e.value().parse::<u8>() { form.write().data1_type_code = v.min(7); }
                    }
                    label { "data1.code(10b):" }
                    input {
                        value: "{form.read().data1_code}",
                        oninput: move |e| if let Ok(v) = e.value().parse::<u16>() { form.write().data1_code = v.min(1023); }
                    }
                    label { "data1.s1/s2/s3:" }
                    div {
                        class: "inline",
                        input {
                            r#type: "checkbox",
                            checked: form.read().data1_s1,
                            onchange: move |_| {
                                let current = form.read().data1_s1;
                                form.write().data1_s1 = !current;
                            }
                        }
                        input {
                            r#type: "checkbox",
                            checked: form.read().data1_s2,
                            onchange: move |_| {
                                let current = form.read().data1_s2;
                                form.write().data1_s2 = !current;
                            }
                        }
                        input {
                            r#type: "checkbox",
                            checked: form.read().data1_s3,
                            onchange: move |_| {
                                let current = form.read().data1_s3;
                                form.write().data1_s3 = !current;
                            }
                        }
                    }
                    label { "data2 dQ/fdQ/JJ/bZ (4b each):" }
                    div { class: "inline",
                        input { value: "{form.read().data2_dq}", oninput: move |e| if let Ok(v)=e.value().parse::<u8>() { form.write().data2_dq = v.min(15); } }
                        input { value: "{form.read().data2_fdq}", oninput: move |e| if let Ok(v)=e.value().parse::<u8>() { form.write().data2_fdq = v.min(15); } }
                        input { value: "{form.read().data2_jj}", oninput: move |e| if let Ok(v)=e.value().parse::<u8>() { form.write().data2_jj = v.min(15); } }
                        input { value: "{form.read().data2_bz}", oninput: move |e| if let Ok(v)=e.value().parse::<u8>() { form.write().data2_bz = v.min(15); } }
                    }
                    label { "data2.sysCode(16b):" }
                    input {
                        value: "{form.read().data2_sys_code}",
                        oninput: move |e| if let Ok(v)=e.value().parse::<u16>() { form.write().data2_sys_code = v; }
                    }
                }
                div { class: "actions",
                    button {
                        onclick: move |_| {
                            let mut logs = logs.clone();
                            let mut packet_hex = packet_hex.clone();
                            let mut bit_desc = bit_desc.clone();
                            let mut server_struct = server_struct.clone();
                            let mut built_packet = built_packet.clone();
                            if let Some(pkt) = build_and_refresh(&form, &mut logs, &mut packet_hex, &mut bit_desc) {
                                server_struct.set(render_server_protocol_table(&form.read(), &pkt));
                                built_packet.set(Some(pkt));
                            }
                        },
                        "生成二进制"
                    }
                    button { onclick: randomize, "自动生成测试数据" }
                    button { onclick: send_full, "UDP 单播一次发送" }
                    button { onclick: send_bits, "准备按位发送(单步模式)" }
                    button { onclick: step_send_one_bit, "单步发送 1 位" }
                    button { onclick: finalize_bit_frame, "完成按位帧并解析" }
                    button { onclick: poll_client, "刷新客户端接收区" }
                }
            }

            div { class: "panel",
                h2 { "服务端结构化编码视图" }
                pre { "{server_struct}" }
            }

            div { class: "panel",
                h2 { "编码结果 / 位域说明（HEX + BIT）" }
                pre { "{packet_hex}" }
                pre { "{bit_desc}" }
            }

            div { class: "panel",
                h2 { "网络传输过程（逐位）" }
                pre { "{tx_flow}" }
            }

            div { class: "panel",
                h2 { "客户端内存缓冲（按位组装成字节）" }
                pre { "{client_mem}" }
            }

            div { class: "panel",
                h2 { "结构体映射过程（memcpy + ntoh + 按声明顺序解包）" }
                pre { "{client_struct}" }
            }

            div { class: "panel",
                h2 { "位落位教学图（接收 bit -> 字节 -> data1/data2 字段）" }
                pre { "{teach_panel}" }
            }

            div { class: "panel",
                h2 { "客户端解析校验（最终语义）" }
                pre { "{verify}" }
            }

            div { class: "panel",
                h2 { "运行日志" }
                ul {
                    for row in logs.read().iter().rev().take(12) {
                        li { "{row}" }
                    }
                }
            }
        }
    }
}

fn build_and_refresh(
    form: &Signal<ProtocolForm>,
    logs: &mut Signal<Vec<String>>,
    packet_hex: &mut Signal<String>,
    bit_desc: &mut Signal<String>,
) -> Option<PacketBuild> {
    match build_packet(&form.read()) {
        Ok(pkt) => {
            packet_hex.set(to_hex(&pkt.packet));
            bit_desc.set(render_bit_layout(&pkt));
            logs.push(format!("编码成功: {} 字节", pkt.packet.len()));
            Some(pkt)
        }
        Err(e) => {
            logs.push(format!("编码失败: {e}"));
            None
        }
    }
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

fn build_packet(form: &ProtocolForm) -> Result<PacketBuild, String> {
    let fc = parse_fc_hex(&form.fc_hex)?;
    // 按“结构体声明顺序”组装主机序位域：先声明的字段占低位，后声明的字段向高位扩展。
    // data1: code, s3, s2, s1, typeCode
    let host_data1 = pack_decl_order_u16(&[
        ("code", 10, form.data1_code as u32),
        ("s3", 1, form.data1_s3 as u32),
        ("s2", 1, form.data1_s2 as u32),
        ("s1", 1, form.data1_s1 as u32),
        ("typeCode", 3, form.data1_type_code as u32),
    ])?;
    // data2: fdQ, dQ, bZ, JJ, sysCode
    let host_data2 = pack_decl_order_u32(&[
        ("fdQ", 4, form.data2_fdq as u64),
        ("dQ", 4, form.data2_dq as u64),
        ("bZ", 4, form.data2_bz as u64),
        ("JJ", 4, form.data2_jj as u64),
        ("sysCode", 16, form.data2_sys_code as u64),
    ])?;
    // 网络传输使用大端字节序，相当于 C++ 里对 packed 结构做 hton。
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

fn parse_packet(packet: &[u8]) -> Result<ParsedPacket, String> {
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
        &[("fdQ", 4), ("dQ", 4), ("bZ", 4), ("JJ", 4), ("sysCode", 16)],
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
        data2_dq: data2_parts[1] as u8,
        data2_fdq: data2_parts[0] as u8,
        data2_jj: data2_parts[3] as u8,
        data2_bz: data2_parts[2] as u8,
        data2_sys_code: data2_parts[4] as u16,
    })
}

fn consume_client_events(events: Vec<ClientEvent>, logs: &mut Signal<Vec<String>>) -> String {
    let mut result = String::new();
    for ev in events {
        match ev {
            ClientEvent::Packet(bytes) => {
                logs.push(format!("客户端收到一次性报文: {} 字节", bytes.len()));
                let _ = writeln!(result, "一次发送报文 HEX: {}", to_hex(&bytes));
                match parse_packet(&bytes) {
                    Ok(p) => {
                        let _ = writeln!(result, "{}", render_parsed(&p));
                        let _ = writeln!(result, "校验: PASS");
                    }
                    Err(e) => {
                        let _ = writeln!(result, "解析失败: {e}");
                        let _ = writeln!(result, "校验: FAIL");
                    }
                }
            }
            ClientEvent::BitFrame(bytes) => {
                logs.push(format!("客户端收到按位重组报文: {} 字节", bytes.len()));
                let _ = writeln!(result, "按位重组报文 HEX: {}", to_hex(&bytes));
                match parse_packet(&bytes) {
                    Ok(p) => {
                        let _ = writeln!(result, "{}", render_parsed(&p));
                        let _ = writeln!(result, "校验: PASS");
                    }
                    Err(e) => {
                        let _ = writeln!(result, "解析失败: {e}");
                        let _ = writeln!(result, "校验: FAIL");
                    }
                }
            }
        }
    }
    result
}

fn render_parsed(p: &ParsedPacket) -> String {
    format!(
        "type=0x{:#04X}, len={}, fc=[{}], data1(typeCode={}, s1={}, s2={}, s3={}, code={}), data2(dQ={}, fdQ={}, JJ={}, bZ={}, sysCode=0x{:04X})",
        p.msg_type,
        p.len,
        p.fc.iter().map(|b| format!("{:02X}", b)).collect::<Vec<_>>().join(" "),
        p.data1_type_code,
        p.data1_s1 as u8,
        p.data1_s2 as u8,
        p.data1_s3 as u8,
        p.data1_code,
        p.data2_dq,
        p.data2_fdq,
        p.data2_jj,
        p.data2_bz,
        p.data2_sys_code
    )
}

fn render_bit_layout(pkt: &PacketBuild) -> String {
    format!(
        "按声明顺序(主机序) data1=code,s3,s2,s1,typeCode => {:016b}\n\
         按声明顺序(主机序) data2=fdQ,dQ,bZ,JJ,sysCode => {:032b}\n\
         转网络序后 data1(be)={:016b}, data2(be)={:032b}\n\
         语义位段(协议定义): data1 [15..13]=typeCode [12]=s1 [11]=s2 [10]=s3 [9..0]=code\n\
         语义位段(协议定义): data2 [31..28]=dQ [27..24]=fdQ [23..20]=JJ [19..16]=bZ [15..0]=sysCode",
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

fn to_hex(bytes: &[u8]) -> String {
    bytes
        .iter()
        .map(|b| format!("{:02X}", b))
        .collect::<Vec<_>>()
        .join(" ")
}

fn bytes_to_bit_chars(bytes: &[u8]) -> Vec<char> {
    let mut out = Vec::with_capacity(bytes.len() * 8);
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
        "传输进度: {sent_bits}/{total_bits} bit\n最新位流(尾部预览):\n{preview}\n\n说明:\n- 每次点击仅发送 1 bit\n- 8 bit 自动组装成 1 字节进入客户端内存缓冲\n- 全部发送后点击“完成按位帧并解析”"
    )
}

fn render_client_memory(bytes: &[u8], raw_bits: &[u8]) -> String {
    let mut s = String::new();
    let bit_text = String::from_utf8_lossy(raw_bits);
    let _ = writeln!(s, "客户端位缓冲区(bit): {}", bit_text);
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
    let _ = writeln!(s, "   - code(10b)={} s3={} s2={} s1={} typeCode={}", form.data1_code, form.data1_s3 as u8, form.data1_s2 as u8, form.data1_s1 as u8, form.data1_type_code);
    let _ = writeln!(s, "5) data2 : 4B  = 0x{:08X}", pkt.data2_raw);
    let _ = writeln!(s, "   - fdQ={} dQ={} bZ={} JJ={} sysCode=0x{:04X}", form.data2_fdq, form.data2_dq, form.data2_bz, form.data2_jj, form.data2_sys_code);
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
            let _ = writeln!(s, "data1 => typeCode={}, s1={}, s2={}, s3={}, code={}", p.data1_type_code, p.data1_s1 as u8, p.data1_s2 as u8, p.data1_s3 as u8, p.data1_code);
            let _ = writeln!(s, "data2 => dQ={}, fdQ={}, JJ={}, bZ={}, sysCode=0x{:04X}", p.data2_dq, p.data2_fdq, p.data2_jj, p.data2_bz, p.data2_sys_code);
            let _ = writeln!(s, "\nStep D: 按结构体声明顺序解包位域 (低位->高位)");
            let _ = writeln!(s, "data1: code -> s3 -> s2 -> s1 -> typeCode");
            let _ = writeln!(s, "data2: fdQ -> dQ -> bZ -> JJ -> sysCode");
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
        let _ = writeln!(s, "等待点击“单步发送 1 位”。");
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
            "  fdQ={} | dQ={} | bZ={} | JJ={} | sysCode=0x{:04X}",
            parsed.data2_fdq, parsed.data2_dq, parsed.data2_bz, parsed.data2_jj, parsed.data2_sys_code
        );
        let _ = writeln!(s, "\n【Step 5: 为什么结构体字段顺序看起来“反着写”】");
        let _ = writeln!(s, "- 协议文档位号是“语义编号”(如 dQ=32..29)");
        let _ = writeln!(s, "- 位域声明顺序是“主机内存落位顺序”(低位字段通常先声明)");
        let _ = writeln!(s, "- 再经过 ntoh/hton 字节序转换，最终与协议语义位号对齐");
    }
    s
}

const APP_STYLE: &str = r#"
body { margin: 0; background: #070b14; color: #d7ecff; font-family: Segoe UI, sans-serif; }
.page { max-width: 1180px; margin: 0 auto; padding: 20px; }
h1 { margin: 0; color: #73d2ff; }
.subtitle { color: #8fb3d9; margin: 8px 0 18px 0; }
.panel {
  border: 1px solid #20364f; border-radius: 12px; padding: 14px;
  margin-bottom: 16px; background: linear-gradient(180deg, #0d1625 0%, #0a1220 100%);
  box-shadow: 0 0 30px rgba(60,150,255,.08) inset;
}
.grid { display: grid; grid-template-columns: 220px 1fr; gap: 10px 12px; align-items: center; }
input { background: #101f35; color: #d7ecff; border: 1px solid #2c4562; border-radius: 6px; padding: 6px; }
.inline { display: flex; gap: 8px; align-items: center; }
.inline input[type='checkbox'] { width: 18px; height: 18px; }
.actions { display: flex; gap: 10px; flex-wrap: wrap; margin-top: 12px; }
button {
  border: 1px solid #2a6d9d; color: #d9f3ff; background: #12314a;
  padding: 8px 12px; border-radius: 8px; cursor: pointer;
}
button:hover { background: #1a4363; }
pre {
  white-space: pre-wrap; word-break: break-word; background: #0a1422;
  border: 1px solid #27425f; border-radius: 8px; padding: 10px;
}
li { margin-bottom: 4px; color: #9bc1e6; }
"#;
