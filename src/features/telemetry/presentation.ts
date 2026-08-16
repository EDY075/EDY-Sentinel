import type { TFunction } from 'i18next'

export function domainValueKey(value: string): string {
  return value.trim().toLocaleLowerCase('en').replaceAll(/[^a-z0-9]+/g, '_').replaceAll(/^_+|_+$/g, '')
}

export function localizedDomainValue(value: string, group: string, t: TFunction): string {
  return t(`${group}.${domainValueKey(value)}`, { defaultValue: value })
}
