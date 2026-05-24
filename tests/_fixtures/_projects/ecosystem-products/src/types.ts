export interface ProductOption {
  label: string;
  value: string;
  disabled?: boolean;
}

export interface ChartPoint {
  month: string;
  value: number;
}

export interface FlowNodeData {
  label: string;
  status: "ready" | "blocked";
}

export interface ApolloProject {
  id: string;
  name: string;
}
