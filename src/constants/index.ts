/** Format: "#RRGGBB" */
export const RGB_TAG_COLOR_PRESETS: readonly {
  label: string;
  color: string;
}[] = [
    { label: "Red", color: "#E5484D" },
    { label: "Orange", color: "#F76B15" },
    { label: "Amber", color: "#F5A524" },
    { label: "Yellow", color: "#E8C60C" },
    { label: "Lime", color: "#A3C93A" },
    { label: "Green", color: "#46A758" },
    { label: "Teal", color: "#12A594" },
    { label: "Cyan", color: "#05A2C2" },
    { label: "Sky", color: "#3E9BE9" },
    { label: "Blue", color: "#3E63DD" },
    { label: "Indigo", color: "#6E56CF" },
    { label: "Violet", color: "#8E4EC6" },
    { label: "Orchid", color: "#C150C1" },
    { label: "Pink", color: "#E54D9E" },
    { label: "Salmon", color: "#F2858F" },
    { label: "Sienna", color: "#A0522D" },
    { label: "Bronze", color: "#AD7F58" },
    { label: "Sand", color: "#8D8D86" },
    { label: "Slate", color: "#5B7083" },
    { label: "Charcoal", color: "#3A3F4B" },
  ];

export const TODO_ESTIMATE_MINUTES_PRESETS: readonly {
  value: number;
}[] = [
    { value: 5 },
    { value: 15 },
    { value: 30 },
    { value: 45 },
    { value: 60 },
    { value: 120 },
    { value: 240 },
    { value: 480 },
  ];

export const TODO_ESTIMATE_POINTS_PRESET: readonly {
  value: number;
}[] = [
    { value: 1 },
    { value: 2 },
    { value: 3 },
    { value: 5 },
    { value: 8 },
    { value: 13 },
    { value: 21 },
    { value: 34 },
    { value: 55 },
    { value: 89 },
  ];
