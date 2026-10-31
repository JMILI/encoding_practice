const $ = (id) => document.getElementById(id);

const state = {
  form: null,
  packetHex: "",
  bitChars: "",
  bitStream: "",
  bitCursor: 0,
  logs: [],
};

/** 与图解教程示例报文完全一致 */
function defaultForm() {
  return {
    msg_type: 0x11,
    fc_hex: "51 73 B8",
    data1_type_code: 5,
    data1_s1: true,
    data1_s2: false,
    data1_s3: true,
    data1_code: 341,
    data2_feild_2: 9,
    data2_feild_1: 3,
    data2_feild_4: 6,
    data2_feild_3: 5,
    data2_sys_code: 4660,
    data3_lo: 5,
    data3_hi: 21,
    data4_info_a: 0x55,
    data4_info_b: 0xaa,
    data4_info_c: 0x155,
  };
}

const EXAMPLE_HEX =
  "11 00 0D 51 73 B8 55 B5 93 65 34 12 AD AB 55 55";

function requireEl(id) {
  const el = $(id);
  if (!el) {
    throw new Error(
      `页面缺少 #${id}。请重启 encoding 服务以加载最新 index.html（stop_all + start_all encoding）`
    );
  }
  return el;
}

function num(id, mask) {
  const v = Number(requireEl(id).value);
  return (Number.isFinite(v) ? v : 0) & mask;
}

function readForm() {
  return {
    msg_type: num("msg_type", 0xff),
    fc_hex: requireEl("fc_hex").value.trim(),
    data1_type_code: num("data1_type_code", 0x7),
    data1_s1: requireEl("data1_s1").checked,
    data1_s2: requireEl("data1_s2").checked,
    data1_s3: requireEl("data1_s3").checked,
    data1_code: num("data1_code", 0x3ff),
    data2_feild_2: num("data2_feild_2", 0xf),
    data2_feild_1: num("data2_feild_1", 0xf),
    data2_feild_4: num("data2_feild_4", 0xf),
    data2_feild_3: num("data2_feild_3", 0xf),
    data2_sys_code: num("data2_sys_code", 0xffff),
    data3_lo: num("data3_lo", 0x7),
    data3_hi: num("data3_hi", 0x1f),
    data4_info_a: num("data4_info_a", 0x7f),
    data4_info_b: num("data4_info_b", 0xff),
    data4_info_c: num("data4_info_c", 0x1ff),
  };
}

function writeForm(form) {
  requireEl("msg_type").value = form.msg_type;
  requireEl("fc_hex").value = form.fc_hex;
  requireEl("data1_type_code").value = form.data1_type_code;
  requireEl("data1_s1").checked = !!form.data1_s1;
  requireEl("data1_s2").checked = !!form.data1_s2;
  requireEl("data1_s3").checked = !!form.data1_s3;
  requireEl("data1_code").value = form.data1_code;
  requireEl("data2_feild_2").value = form.data2_feild_2;
  requireEl("data2_feild_1").value = form.data2_feild_1;
  requireEl("data2_feild_4").value = form.data2_feild_4;
  requireEl("data2_feild_3").value = form.data2_feild_3;
  requireEl("data2_sys_code").value = form.data2_sys_code;
  requireEl("data3_lo").value = form.data3_lo ?? 0;
  requireEl("data3_hi").value = form.data3_hi ?? 0;
  requireEl("data4_info_a").value = form.data4_info_a ?? 0;
  requireEl("data4_info_b").value = form.data4_info_b ?? 0;
  requireEl("data4_info_c").value = form.data4_info_c ?? 0;
  updateExampleBanner();
}

function updateExampleBanner() {
  const el = $("example_hex_banner");
  if (!el) return;
  const cur = (state.packetHex || "").trim().toUpperCase();
  const ok = cur === EXAMPLE_HEX;
  el.textContent = ok
    ? `当前编码与图解示例一致：${EXAMPLE_HEX}`
    : `图解示例报文：${EXAMPLE_HEX}${cur ? ` ｜ 当前：${cur}` : " ｜ 尚未编码"}`;
  el.style.borderColor = ok ? "#2d6a4f" : "#2a5080";
}

function setPre(id, text) {
  const el = $(id);
  if (el) el.textContent = text || "";
}

function pushLog(msg) {
  state.logs.unshift(msg);
  state.logs = state.logs.slice(0, 16);
  const ul = $("logs");
  if (!ul) return;
  ul.innerHTML = "";
  for (const row of state.logs) {
    const li = document.createElement("li");
    li.textContent = row;
    ul.appendChild(li);
  }
}

function showError(msg) {
  const el = $("error");
  if (el) el.textContent = msg || "";
}

function clearBitMode(reason) {
  state.bitChars = "";
  state.bitStream = "";
  state.bitCursor = 0;
  setBtnState();
  if (reason) pushLog(reason);
}

function setBtnState() {
  const hasBits = !!state.bitChars;
  const step = $("btn_step_bit");
  const fin = $("btn_finalize");
  if (step) step.disabled = !hasBits;
  if (fin) fin.disabled = !state.bitStream;
}

async function post(path, body) {
  let res;
  try {
    res = await fetch(path, {
      method: "POST",
      headers: { "Content-Type": "application/json" },
      body: JSON.stringify(body ?? {}),
    });
  } catch (e) {
    throw new Error(`网络失败：${e.message}（请确认 encoding 服务已启动）`);
  }
  const text = await res.text();
  let data = null;
  try {
    data = text ? JSON.parse(text) : null;
  } catch {
    // Axum 反序列化失败等会返回纯文本 "Failed to deserialize..."
    throw new Error(
      text
        ? text.slice(0, 300)
        : `请求失败 HTTP ${res.status}（响应非 JSON，请重启 encoding 服务）`
    );
  }
  if (!res.ok) {
    throw new Error((data && data.error) || `请求失败: ${res.status}`);
  }
  return data;
}

function applyBuild(data) {
  state.packetHex = data.packet_hex || "";
  state.form = readForm();
  setPre("packet_hex", data.packet_hex);
  setPre("bit_desc", data.bit_desc);
  setPre("server_struct", data.server_struct);
  updateExampleBanner();
  pushLog(`编码成功: ${data.packet_len} 字节 → ${data.packet_hex}`);
}

async function onBuild() {
  showError("");
  try {
    clearBitMode();
    const form = readForm();
    const data = await post("/api/build", form);
    applyBuild(data);
  } catch (e) {
    showError(e.message);
    pushLog(`编码失败: ${e.message}`);
  }
}

async function onResetExample() {
  showError("");
  try {
    writeForm(defaultForm());
    clearBitMode("已恢复图解默认报文");
    await onBuild();
  } catch (e) {
    showError(e.message);
    pushLog(e.message);
  }
}

async function onRandom() {
  showError("");
  try {
    const form = await post("/api/random", {});
    writeForm(form);
    clearBitMode("表单已随机，按位发送状态已清空");
    const data = await post("/api/build", form);
    applyBuild(data);
    setPre("tx_flow", "");
    setPre("client_mem", "");
    setPre("client_struct", "");
    setPre("teach_panel", "");
    setPre("verify", "");
    pushLog("已生成随机测试数据并同步编码");
  } catch (e) {
    showError(e.message);
    pushLog(e.message);
  }
}

async function onSimulateFull() {
  showError("");
  try {
    const form = readForm();
    const build = await post("/api/build", form);
    applyBuild(build);
    clearBitMode();
    const sim = await post("/api/simulate-full", form);
    setPre("tx_flow", sim.tx_flow);
    setPre("client_mem", sim.client_mem);
    setPre("client_struct", sim.client_struct);
    setPre("teach_panel", sim.teach_panel);
    setPre("verify", sim.verify);
    pushLog("模拟 UDP 一次发送完成（已用当前表单重新编码）");
  } catch (e) {
    showError(e.message);
    pushLog(e.message);
  }
}

async function onPrepareBits() {
  showError("");
  try {
    const form = readForm();
    const data = await post("/api/prepare-bits", form);
    applyBuild(data.build);
    state.bitChars = data.bit_chars;
    state.bitStream = "";
    state.bitCursor = 0;
    setBtnState();
    setPre("tx_flow", data.tx_flow);
    setPre("client_mem", "客户端内存区为空，等待位流组装成完整字节。");
    setPre("client_struct", "尚未完成完整报文，结构体无法映射。");
    setPre("teach_panel", "");
    setPre("verify", "");
    pushLog(`已准备按位发送，共 ${data.total_bits} bit（基于当前表单）`);
  } catch (e) {
    showError(e.message);
    pushLog(e.message);
  }
}

async function onStepBit() {
  showError("");
  if (!state.bitChars) {
    showError("请先点击「准备按位发送」");
    return;
  }
  if (state.bitCursor >= state.bitChars.length) {
    pushLog("所有位已发送，请点击「完成按位帧并解析」");
    setBtnState();
    return;
  }
  state.bitStream += state.bitChars[state.bitCursor];
  state.bitCursor += 1;
  setBtnState();
  try {
    const data = await post("/api/step-preview", {
      packet_hex: state.packetHex,
      bit_stream: state.bitStream,
    });
    setPre("tx_flow", data.tx_flow);
    setPre("client_mem", data.client_mem);
    setPre("teach_panel", data.teach_panel);
    pushLog(`已发送第 ${state.bitCursor}/${state.bitChars.length} bit`);
  } catch (e) {
    showError(e.message);
  }
}

async function onFinalizeBits() {
  showError("");
  if (!state.bitStream) {
    showError("尚未发送任何位");
    return;
  }
  if (!state.packetHex) {
    showError("缺少编码报文，请先「准备按位发送」");
    return;
  }
  try {
    const data = await post("/api/finalize-bits", {
      packet_hex: state.packetHex,
      bit_stream: state.bitStream,
    });
    setPre("client_mem", data.client_mem);
    setPre("client_struct", data.client_struct);
    setPre("teach_panel", data.teach_panel);
    setPre("verify", data.verify);
    pushLog("按位传输完成");
  } catch (e) {
    showError(e.message);
    pushLog(e.message);
  }
}

function onFormInput() {
  if (state.bitChars || state.bitStream) {
    clearBitMode("检测到表单修改，已清空按位发送进度（请重新准备）");
  }
}

function bindEvents() {
  $("btn_build").addEventListener("click", onBuild);
  $("btn_random").addEventListener("click", onRandom);
  $("btn_full").addEventListener("click", onSimulateFull);
  $("btn_prepare_bits").addEventListener("click", onPrepareBits);
  $("btn_step_bit").addEventListener("click", onStepBit);
  $("btn_finalize").addEventListener("click", onFinalizeBits);
  const resetBtn = $("btn_reset_example");
  if (resetBtn) resetBtn.addEventListener("click", onResetExample);

  const panel = document.querySelector(".panel .grid");
  if (panel) {
    panel.addEventListener("input", onFormInput);
    panel.addEventListener("change", onFormInput);
  }
  setBtnState();
}

window.addEventListener("DOMContentLoaded", () => {
  try {
    writeForm(defaultForm());
    bindEvents();
    pushLog("Web 版已就绪；默认数据与图解教程示例报文一致");
    onBuild();
  } catch (e) {
    showError(e.message);
    pushLog(e.message);
  }
});
