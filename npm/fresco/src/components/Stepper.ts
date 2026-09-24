/**
 * Stepper Component - Step indicator for wizards
 */

import { defineComponent, h, type PropType } from "vue";

export interface Step {
  key: string;
  label: string;
  description?: string;
}

export type StepStatus = "pending" | "current" | "completed" | "error";

export interface StepperProps {
  /** Steps */
  steps: Step[];
  /** Current step index */
  current?: number;
  /** Completed steps (indices) */
  completed?: number[];
  /** Error steps (indices) */
  errors?: number[];
  /** Direction */
  direction?: "horizontal" | "vertical";
  /** Show step numbers */
  showNumbers?: boolean;
  /** Completed icon */
  completedIcon?: string;
  /** Error icon */
  errorIcon?: string;
  /** Current icon */
  currentIcon?: string;
  /** Pending icon */
  pendingIcon?: string;
}

export const Stepper = defineComponent({
  name: "Stepper",
  props: {
    steps: {
      type: Array as PropType<Step[]>,
      required: true,
    },
    current: {
      type: Number,
      default: 0,
    },
    completed: {
      type: Array as PropType<number[]>,
      default: () => [],
    },
    errors: {
      type: Array as PropType<number[]>,
      default: () => [],
    },
    direction: {
      type: String as PropType<"horizontal" | "vertical">,
      default: "horizontal",
    },
    showNumbers: {
      type: Boolean,
      default: true,
    },
    completedIcon: {
      type: String,
      default: "✓",
    },
    errorIcon: {
      type: String,
      default: "✗",
    },
    currentIcon: {
      type: String,
      default: "●",
    },
    pendingIcon: {
      type: String,
      default: "○",
    },
  },
  setup(props) {
    const getStatus = (index: number): StepStatus => {
      if (props.errors?.includes(index)) return "error";
      if (props.completed?.includes(index)) return "completed";
      if (index === props.current) return "current";
      return "pending";
    };

    const getIcon = (index: number, status: StepStatus): string => {
      if (props.showNumbers && status === "pending") {
        return String(index + 1);
      }
      switch (status) {
        case "completed":
          return props.completedIcon;
        case "error":
          return props.errorIcon;
        case "current":
          return props.currentIcon;
        default:
          return props.pendingIcon;
      }
    };

    const getColor = (status: StepStatus): string => {
      switch (status) {
        case "completed":
          return "green";
        case "error":
          return "red";
        case "current":
          return "cyan";
        default:
          return "gray";
      }
    };

    return () => {
      const isHorizontal = props.direction === "horizontal";
      const connector = isHorizontal ? "───" : "│";

      const children = props.steps.flatMap((step, index) => {
        const status = getStatus(index);
        const icon = getIcon(index, status);
        const color = getColor(status);
        const isLast = index === props.steps.length - 1;

        const stepContent = [
          h(
            "box",
            {
              key: `step-${step.key}`,
              style: {
                flex_direction: isHorizontal ? "column" : "row",
                align_items: "center",
              },
            },
            [
              h(
                "text",
                {
                  fg: color,
                  bold: status === "current",
                },
                `[${icon}]`,
              ),
              h(
                "text",
                {
                  fg: status === "current" ? "white" : "gray",
                  bold: status === "current",
                  style: isHorizontal ? { margin_top: 0.5 } : { margin_left: 1 },
                },
                step.label,
              ),
            ],
          ),
        ];

        if (!isLast) {
          stepContent.push(
            h(
              "text",
              {
                key: `connector-${index}`,
                dim: true,
                style: isHorizontal
                  ? { margin_left: 1, margin_right: 1 }
                  : { margin_top: 0.5, margin_bottom: 0.5, margin_left: 1 },
              },
              connector,
            ),
          );
        }

        return stepContent;
      });

      return h(
        "box",
        {
          style: {
            flex_direction: isHorizontal ? "row" : "column",
            align_items: isHorizontal ? "flex-start" : "stretch",
          },
        },
        children,
      );
    };
  },
});
