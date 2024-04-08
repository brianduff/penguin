export interface ServiceStatus {
  active: ActiveState
}

export enum ActiveState {
  ACTIVE = "Active",
  DEACTIVATING = "Deactivating",
  INACTIVE = "Inactive",
  UNKNOWN = "Unknown"
}