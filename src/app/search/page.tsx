"use client";

import {
  Command,
  CommandEmpty,
  CommandInput,
  CommandList,
} from "@/components/ui/command";
// import { Calculator, Calendar, Smile } from "lucide-react";
import { invoke } from "@tauri-apps/api/core";
import { useEffect, useRef, useState } from "react";
import { listen } from "@tauri-apps/api/event";

type CatchError = {
  OtherError: string;
};

export default function Search() {
  const [searchValue, setSearchValue] = useState<string>("");
  const [app, setApp] = useState<string[]>([]);
  const timeoutRef = useRef<NodeJS.Timeout | null>(null);

  /**
   * 下面的代码的功能是监听#search的size变化，并将变化后的size传后rust端
   *  rust端根据传入的size修改窗口的大小
   *
   *  修改窗口大小是为了避免前端页面变小时，出现一片透明的窗口区域，在这片区域内，鼠标因没有透传而无法点击的问题
   */
  useEffect(() => {
    const searchDiv = document.querySelector("#search") as HTMLDivElement;
    const resizeObserver = new ResizeObserver(async (entries) => {
      if (!Array.isArray(entries) || !entries.length) return;
      for (const element of entries) {
        await invoke("window_resize", {
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
    listen("hidewindow", () => {
      setApp([]);
      setSearchValue("");
    });
  };

  listenInput();

  /**
   * @description 根据参数返回对应的搜索结果
   * @param searchValue 搜索的值
   */
  const search = async (searchValue: string) => {
    if (timeoutRef.current !== null) {
      clearTimeout(timeoutRef.current);
    }
    timeoutRef.current = setTimeout(async () => {
      if (typeof window === "undefined") return;
      try {
        const result: string[] = await invoke("get_search_result", {
          searchValue,
        });
        console.log("result: ", result);
        setApp(result);
      } catch (error) {
        console.log("error: ", (error as CatchError).OtherError);
      }
    }, 500);
  };

  useEffect(() => {
    search(searchValue);
  }, [searchValue]);

  const executeApp = async (appName: string) => {
    if (typeof window === "undefined") return;
    await invoke("execute_app", { appName });
  };

  return (
    <Command
      id="search"
      className="rounded-lg bordfer shadow-mdf md:min-w-[450px]"
    >
      <CommandInput
        placeholder="输入软件名称"
        value={searchValue}
        onChangeCapture={(e) => setSearchValue(e.currentTarget.value)}
      />
      <CommandList>
        {app.length > 0 ? (
          app.map((item) => (
            <div
              className=" my-2"
              key={item}
            >
              <div
                className="h-full w-full pl-4 py-2 cursor-pointer hover:bg-gray-100"
                onClick={() => executeApp(item)}
              >
                {item}
              </div>
            </div>
          ))
        ) : (
          <CommandEmpty>没有找到app。</CommandEmpty>
        )}
      </CommandList>
    </Command>
  );
}
