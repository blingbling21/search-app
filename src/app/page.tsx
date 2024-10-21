"use client";

import { invoke } from "@tauri-apps/api/core";

export default function Home() {
  const click = () => {
    if (typeof window === "undefined") return;
    invoke("test");
  }
  return <button onClick={() => click()}>打开搜索框</button>;
}
