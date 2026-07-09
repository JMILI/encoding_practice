const $ = (id) => document.getElementById(id);

const state = {
  form: null,
  packetHex: "",
  bitChars: "",
  bitStream: "",
  bitCursor: 0,
  logs: [],
};

function defaultForm() {
  return {
    msg_type: 17,
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
  };
}

function readForm() {
  return {
    msg_type: Number($("msg_type").value) & 0xff,
    fc_hex: $("fc_hex").value.trim(),
    data1_type_code: Number($("data1_type_code").value) & 0x7,
    data1_s1: $("data1_s1").checked,
    data1_s2: $("data1_s2").checked,
    data1_s3: $("data1_s3").checked,
    data1_code: Number($("data1_code").value) & 0x3ff,
    data2_feild_2: Number($("data2_feild_2").value) & 0xf,
    data2_feild_1: Number($("data2_feild_1").value) & 0xf,
    data2_feild_4: Number($("data2_feild_4").value) & 0xf,
    data2_feild_3: Number($("data2_feild_3").value) & 0xf,
    data2_sys_code: Number($("data2_sys_code").value) & 0xffff,
  };
}

function writeForm(form) {
  $("msg_type").value = form.msg_type;
  $("fc_hex").value = form.fc_hex;
  $("data1_type_code").value = form.data1_type_code;
  $("data1_s1").checked = form.data1_s1;
  $("data1_s2").checked = form.data1_s2;
  $("data1_s3").checked = form.data1_s3;
  $("data1_code").value = form.data1_code;
  $("data2_feild_2").value = form.data2_feild_2;
  $("data2_feild_1").value = form.data2_feild_1;
  $("data2_feild_4").value = form.data2_feild_4;
  $("data2_feild_3").value = form.data2_feild_3;
  $("data2_sys_code").value = form.data2_sys_code;
}

function setPre(id, text) {
  $(id).textContent = text || "";
}

function pushLog(msg) {
  state.logs.unshift(msg);
  state.logs = state.logs.slice(0, 12);
  const ul = $("logs");
  ul.innerHTML = "";
  for (const row of state.logs) {
    const li = document.createElement("li");
    li.textContent = row;
    ul.appendChild(li);
  }
}

function showError(msg) {
  $("error").textContent = msg || "";
}

async function post(path, body) {
  const res = await fetch(path, {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify(body),
  });
  const data = await res.json();
  if (!res.ok) {
    throw new Error(data.error || `请求失败: ${res.status}`);
  }
  return data;
}

function applyBuild(data) {
  state.packetHex = data.packet_hex;
  setPre("packet_hex", data.packet_hex);
  setPre("bit_desc", data.bit_desc);
  setPre("server_struct", data.server_struct);
  pushLog(`编码成功: ${data.packet_len} 字节`);
}

async function onBuild() {
  showError("");
  try {
    const form = readForm();
    const data = await post("/api/build", form);
    applyBuild(data);
  } catch (e) {
    showError(e.message);
    pushLog(`编码失败: ${e.message}`);
  }
}

async function onRandom() {
  showError("");
  try {
    const form = await post("/api/random", {});
    writeForm(form);
    pushLog("已生成随机测试数据");
  } catch (e) {
    showError(e.message);
  }
}

async function onSimulateFull() {
  showError("");
  try {
    const form = readForm();
    const build = await post("/api/build", form);
    applyBuild(build);
    const sim = await post("/api/simulate-full", form);
    setPre("tx_flow", sim.tx_flow);
    setPre("client_mem", sim.client_mem);
    setPre("client_struct", sim.client_struct);
    setPre("teach_panel", sim.teach_panel);
    setPre("verify", sim.verify);
    pushLog("模拟 UDP 一次发送完成");
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
    setPre("tx_flow", data.tx_flow);
    setPre("client_mem", "客户端内存区为空，等待位流组装成完整字节。");
    setPre("client_struct", "尚未完成完整报文，结构体无法映射。");
    setPre("teach_panel", "");
    setPre("verify", "");
    pushLog(`已准备按位发送，共 ${data.total_bits} bit`);
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
    return;
  }
  state.bitStream += state.bitChars[state.bitCursor];
  state.bitCursor += 1;
  try {
    const data = await post("/api/step-preview", {
      packet_hex: state.packetHex,
      bit_stream: state.bitStream,
    });
    setPre("tx_flow", data.tx_flow);
    setPre("client_mem", data.client_mem);
    setPre("teach_panel", data.teach_panel);
    pushLog(`已发送第 ${state.bitCursor} bit`);
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

function bindEvents() {
  $("btn_build").addEventListener("click", onBuild);
  $("btn_random").addEventListener("click", onRandom);
  $("btn_full").addEventListener("click", onSimulateFull);
  $("btn_prepare_bits").addEventListener("click", onPrepareBits);
  $("btn_step_bit").addEventListener("click", onStepBit);
  $("btn_finalize").addEventListener("click", onFinalizeBits);
}

window.addEventListener("DOMContentLoaded", () => {
  writeForm(defaultForm());
  bindEvents();
  pushLog("Web 版已就绪，无需 GTK/WebView 依赖");
});
