/// <reference types="@raycast/api">

/* 🚧 🚧 🚧
 * This file is auto-generated from the extension's manifest.
 * Do not modify manually. Instead, update the `package.json` file.
 * 🚧 🚧 🚧 */

/* eslint-disable @typescript-eslint/ban-types */

type ExtensionPreferences = {}

/** Preferences accessible in all the extension's commands */
declare type Preferences = ExtensionPreferences

declare namespace Preferences {
  /** Preferences accessible in the `index` command */
  export type Index = ExtensionPreferences & {
  /** Preferred Format - Default date format used in the copy action when ⏎ is pressed */
  "defaultFormat": "unix-ms" | "unix-s" | "iso" | "human",
  /** Secondary Format - Secondary date format used in the copy action when ⌘ + ⏎ is pressed */
  "secondaryFormat": "unix-ms" | "unix-s" | "iso" | "human",
  /** Copy Action - Action to take when copying the converted date */
  "copyAction": "copy" | "paste" | "both",
  /** Default Display Time Format - Default time format for displaying time. By default, 'dddd, MMMM D, YYYY hh:mm:ss' will be used. Available formats can be found in the day.js library documentation. */
  "displayFormat": string,
  /** Human-readable Time Format - Time format for human-readable time. By default, 'dddd, MMMM D, YYYY hh:mm:ss' will be used. Available formats can be found in the day.js library documentation. */
  "humanFormat": string
}
}

declare namespace Arguments {
  /** Arguments passed to the `index` command */
  export type Index = {}
}

