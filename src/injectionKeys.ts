import { InjectionKey } from 'vue'

export type Alert = (error: Error) => void
export const iAlert: InjectionKey<Alert> = Symbol('alert')
