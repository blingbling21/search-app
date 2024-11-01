"use client";

import {
  Command,
  CommandEmpty,
  CommandGroup,
  CommandInput,
  CommandItem,
  CommandList,
  CommandSeparator,
  CommandShortcut,
} from "@/components/ui/command";
import {
  Calculator,
  Calendar,
  CreditCard,
  Settings,
  Smile,
  User,
} from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useState } from "react";
import { listen } from "@tauri-apps/api/event";

export default function Search() {

  // 设置ResizeObserver监听页面的size变化，并将最新的size传给rust更新窗口的size。
  useEffect(() => {
    const searchDiv = document.querySelector("#search") as HTMLDivElement;
    const resizeObserver = new ResizeObserver(async (entries) => {
      if (!Array.isArray(entries) || !entries.length) return;
      for (const element of entries) {
        await invoke("set_window_size", {
          width: element.contentRect.width,
          height: element.contentRect.height,
        });
      }
    });
    resizeObserver.observe(searchDiv);
    return () => {
      resizeObserver.unobserve(searchDiv);
    };
  }, []);

  // 监听rust的search-focus事件，事件触发时搜索框focus。
  const listenInput = () => {
    if (typeof window === "undefined") return;
    listen("search-focus", () => {
      const searchInput = document.querySelector("input") as HTMLInputElement;
      console.log("searchInput: ", searchInput);
      searchInput.focus();
    });
  }

  listenInput();

  const search = async (searchValue: string) => {
    if (typeof window === "undefined") return;
    await invoke("send_search_result", { searchValue });
  }

  return (
    <Command
      id="search"
      className="rounded-lg bordfer shadow-mdf md:min-w-[450px]"
    >
      <CommandInput placeholder="输入搜索软件名称..." onChangeCapture={(e) => search(e.currentTarget.value)} />
      <CommandList>
        <CommandEmpty>No results found.</CommandEmpty>
        <CommandGroup heading="Suggestions">
          <CommandItem>
            <Calendar />
            <span>Calendar</span>
          </CommandItem>
          <CommandItem>
            <Smile />
            <span>Search Emoji</span>
          </CommandItem>
          <CommandItem disabled>
            <Calculator />
            <span>Calculator</span>
          </CommandItem>
        </CommandGroup>
        <CommandSeparator />
        <CommandGroup heading="Settings">
          <CommandItem>
            <User />
            <span>Profile</span>
            <CommandShortcut>⌘P</CommandShortcut>
          </CommandItem>
          <CommandItem>
            <CreditCard />
            <span>Billing</span>
            <CommandShortcut>⌘B</CommandShortcut>
          </CommandItem>
          <CommandItem>
            <Settings />
            <span>Settings</span>
            <CommandShortcut>⌘S</CommandShortcut>
          </CommandItem>
        </CommandGroup>
      </CommandList>
    </Command>
  );
}
