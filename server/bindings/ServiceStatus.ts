export interface ServiceStatus {
  active: ActiveState
}

export enum ActiveState {
  ACTIVE = "Active",
  Deactivating = "Deactivating",
  Inactive = "Inactive",
  Unknown = "Unknown"
}