// Generated from tools/config/pkl/package-tasks/npm/builder/unplugin/tasks.pkl
Object.defineProperty(exports, Symbol.toStringTag, { value: "Module" });
//#region \0rolldown/runtime.js
var __create = Object.create;
var __defProp = Object.defineProperty;
var __getOwnPropDesc = Object.getOwnPropertyDescriptor;
var __getOwnPropNames = Object.getOwnPropertyNames;
var __getProtoOf = Object.getPrototypeOf;
var __hasOwnProp = Object.prototype.hasOwnProperty;
var __copyProps = (to, from, except, desc) => {
	if (from && typeof from === "object" || typeof from === "function") for (var keys = __getOwnPropNames(from), i = 0, n = keys.length, key; i < n; i++) {
		key = keys[i];
		if (!__hasOwnProp.call(to, key) && key !== except) __defProp(to, key, {
			get: ((k) => from[k]).bind(null, key),
			enumerable: !(desc = __getOwnPropDesc(from, key)) || desc.enumerable
		});
	}
	return to;
};
var __toESM = (mod, isNodeMode, target) => (target = mod != null ? __create(__getProtoOf(mod)) : {}, __copyProps(isNodeMode || !mod || !mod.__esModule ? __defProp(target, "default", {
	value: mod,
	enumerable: true
}) : target, mod));
//#endregion
let node_child_process = require("node:child_process");
let node_fs = require("node:fs");
node_fs = __toESM(node_fs, 1);
let node_os = require("node:os");
let node_path = require("node:path");
node_path = __toESM(node_path, 1);
//#region tools/support/command-invocation.ts
function windowsInvocation(command, args) {
	if (process.platform !== "win32") return {
		command,
		args: [...args],
		verbatim: false
	};
	let executable = command;
	if (!node_path.default.extname(executable)) {
		const directories = command.includes("/") || command.includes("\\") ? [""] : (process.env.PATH ?? "").split(node_path.default.delimiter);
		const extensions = (process.env.PATHEXT ?? ".COM;.EXE;.BAT;.CMD").split(";");
		outer: for (const directory of directories) for (const extension of extensions) {
			const candidate = node_path.default.join(directory, command + extension.toLowerCase());
			if (node_fs.default.statSync(candidate, { throwIfNoEntry: false })?.isFile()) {
				executable = candidate;
				break outer;
			}
		}
	}
	if (!/\.(cmd|bat)$/i.test(executable)) return {
		command: executable,
		args: [...args],
		verbatim: false
	};
	const doubleEscape = /node_modules[\\/]\.bin[\\/]/i.test(executable);
	const escaped = args.map((arg) => escapeWindowsArgument(arg, doubleEscape));
	const line = [node_path.default.normalize(executable).replace(WINDOWS_META, "^$1"), ...escaped].join(" ");
	return {
		command: process.env.ComSpec ?? "cmd.exe",
		args: [
			"/d",
			"/s",
			"/c",
			`"${line}"`
		],
		verbatim: true
	};
}
/*! Argument escaping adapted from cross-spawn (MIT), Copyright (c) 2018 Made With
* MOXY Lda <hello@moxy.studio>. Permission is hereby granted, free of charge, to
* any person obtaining a copy of this software and associated documentation
* files (the "Software"), to deal in the Software without restriction, including
* without limitation the rights to use, copy, modify, merge, publish, distribute,
* sublicense, and/or sell copies of the Software, and to permit persons to whom
* the Software is furnished to do so, subject to the following conditions:
* The above copyright notice and this permission notice shall be included in
* all copies or substantial portions of the Software. THE SOFTWARE IS PROVIDED
* "AS IS", WITHOUT WARRANTY OF ANY KIND, EXPRESS OR IMPLIED, INCLUDING BUT NOT
* LIMITED TO THE WARRANTIES OF MERCHANTABILITY, FITNESS FOR A PARTICULAR
* PURPOSE AND NONINFRINGEMENT. IN NO EVENT SHALL THE AUTHORS OR COPYRIGHT
* HOLDERS BE LIABLE FOR ANY CLAIM, DAMAGES OR OTHER LIABILITY, WHETHER IN AN
* ACTION OF CONTRACT, TORT OR OTHERWISE, ARISING FROM, OUT OF OR IN CONNECTION
* WITH THE SOFTWARE OR THE USE OR OTHER DEALINGS IN THE SOFTWARE.
*/
var WINDOWS_META = /([()\][%!^"`<>&|;, *?])/g;
function escapeWindowsArgument(value, twice) {
	const escaped = `"${value.replace(/(?=(\\+?)?)\1"/g, "$1$1\\\"").replace(/(?=(\\+?)?)\1$/, "$1$1")}"`.replace(WINDOWS_META, "^$1");
	return twice ? escaped.replace(WINDOWS_META, "^$1") : escaped;
}
//#endregion
//#region tools/support/tasks/package-task.ts
function argumentValues(command, cwd) {
	return (command.arguments ?? []).flatMap((argument) => {
		if (command.absolutePaths?.includes(argument)) return [node_path.default.resolve(cwd, argument)];
		if (process.platform === "win32" || !command.globs?.includes(argument)) return [argument];
		const files = (0, node_fs.globSync)(argument.split("/").map((part) => part === "**" ? "*" : part).join("/"), { cwd }).sort((left, right) => left < right ? -1 : left > right ? 1 : 0);
		return files.length ? files : [argument];
	});
}
function runPackageTask(tasks, argv, environment = process.env) {
	const [name, ...forwarded] = argv;
	if (!name || !Object.hasOwn(tasks, name)) throw new Error("Unknown package task: " + name);
	const commands = tasks[name];
	for (const [index, command] of commands.entries()) {
		const cwd = (0, node_fs.realpathSync)(command.directory ?? ".");
		const env = {
			...environment,
			...command.environment
		};
		for (const [key, value] of Object.entries(command.defaults ?? {})) if (!env[key]) env[key] = value;
		const args = argumentValues(command, cwd);
		if (index === commands.length - 1) args.push(...forwarded);
		const invocation = windowsInvocation(command.program, args);
		const result = (0, node_child_process.spawnSync)(invocation.command, invocation.args, {
			cwd,
			env,
			stdio: "inherit",
			windowsVerbatimArguments: invocation.verbatim
		});
		if (result.error) {
			console.error(result.error.message);
			return result.error.code === "ENOENT" ? 127 : 126;
		}
		if (result.signal) return 128 + node_os.constants.signals[result.signal];
		if (result.status !== 0) return result.status ?? 1;
	}
	return 0;
}
//#endregion
exports.runPackageTask = runPackageTask;

const tasks = {
  "test": [
    {
      "program": "vp",
      "arguments": [
        "pack"
      ]
    },
    {
      "program": "node",
      "arguments": [
        "--test",
        "src/*.test.ts",
        "src/**/*.test.ts"
      ],
      "globs": [
        "src/*.test.ts",
        "src/**/*.test.ts"
      ]
    }
  ]
};
exports.tasks = tasks;
if (require.main === module) process.exitCode = exports.runPackageTask(tasks, process.argv.slice(2));
