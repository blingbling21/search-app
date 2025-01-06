"use client";

import { Button } from "@/components/ui/button";
import { Input } from "@/components/ui/input";
import { Label } from "@/components/ui/label";
import { useEffect, useState } from "react";
import {
  AlertDialog,
  AlertDialogAction,
  AlertDialogCancel,
  AlertDialogContent,
  AlertDialogDescription,
  AlertDialogFooter,
  AlertDialogHeader,
  AlertDialogTitle,
  AlertDialogTrigger,
} from "@/components/ui/alert-dialog";
import { invoke } from "@tauri-apps/api/core";

export default function Setting() {
  const [canModify, setCanModify] = useState<boolean>(false);
  const [appPath, setAppPath] = useState<string>("");

  const initAppPath = async () => {
    const initPath = await invoke<string>("get_app_path");
    console.log("initPath: ", initPath);
    setAppPath(initPath);
  }

  useEffect(() => {
    initAppPath();
  }, []);

  const save = () => {
    setCanModify(false);
  };

  const modify = async () => {
    setCanModify(true);
  };

  const chooseFolder = async () => {
    const path = await invoke<string>("choose_dir");
    console.log("path: ", path);
    setAppPath(path);
  }

  return (
    <div className="p-5">
      <div className=" flex items-center">
        <Label className="w-16">app路径</Label>
        <Input className="w-96" disabled={!canModify} value={appPath} onClick={chooseFolder} />
        {canModify ? (
          <AlertDialog>
            <AlertDialogTrigger asChild>
              <Button variant="default" className="ml-2">保存</Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>
                  确定将app启动文件夹的路径改为下面的路径吗？
                </AlertDialogTitle>
                <AlertDialogDescription>
                  {appPath}
                </AlertDialogDescription>
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>取消</AlertDialogCancel>
                <AlertDialogAction onClick={save}>确定</AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        ) : (
          <AlertDialog>
            <AlertDialogTrigger asChild>
              <Button variant="default" className="ml-2">修改</Button>
            </AlertDialogTrigger>
            <AlertDialogContent>
              <AlertDialogHeader>
                <AlertDialogTitle>
                  确定要修改app启动文件夹的路径吗？
                </AlertDialogTitle>
                {/* <AlertDialogDescription>
                  This action cannot be undone. This will permanently delete
                  your account and remove your data from our servers.
                </AlertDialogDescription> */}
              </AlertDialogHeader>
              <AlertDialogFooter>
                <AlertDialogCancel>取消</AlertDialogCancel>
                <AlertDialogAction onClick={modify}>确定</AlertDialogAction>
              </AlertDialogFooter>
            </AlertDialogContent>
          </AlertDialog>
        )}
      </div>
    </div>
  );
}
