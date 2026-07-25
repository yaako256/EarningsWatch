import { api } from './client'
import type { User } from '../types/api'

export function login(username: string, password: string) {
  return api.login(username, password)
}

export function logout() {
  return api.post<null>('/auth/logout')
}

export function fetchCurrentUser() {
  return api.get<User>('/auth/me')
}
