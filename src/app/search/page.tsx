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
import { useEffect } from "react";
import { listen } from "@tauri-apps/api/event";

export default function Search() {
  useEffect(() => {
    const searchDiv = document.querySelector("#search") as HTMLDivElement;
    const resizeObserver = new ResizeObserver(async (entries) => {
      if (!Array.isArray(entries) || !entries.length) return;
      for (const element of entries) {
        await invoke("test", {
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

  const listenInput = () => {
    if (typeof window === "undefined") return;
    listen("search-focus", () => {
      const searchInput = document.querySelector("input") as HTMLInputElement;
      console.log("searchInput: ", searchInput);
      searchInput.focus();
    });
  }

  listenInput();

  return (
    <Command
      id="search"
      className="rounded-lg bordfer shadow-mdf md:min-w-[450px]"
    >
      <CommandInput placeholder="Type a command or search..." />
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
