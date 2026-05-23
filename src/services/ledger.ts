import { invoke } from './ipc';

export interface LedgerCategoryDto {
  id: string;
  name: string;
  parentId: string | null;
  sortOrder: number;
  createdAt: string | null;
  updatedAt: string | null;
}

export interface LedgerTransactionDto {
  id: string;
  amount: string;
  currency: string;
  direction: string;
  categoryId: string | null;
  categoryName: string | null;
  occurredOn: string;
  note: string | null;
  createdAt: string | null;
  updatedAt: string | null;
}

export interface LedgerListFilter {
  dateFrom?: string;
  dateTo?: string;
  /** all | in | out */
  direction?: string;
  categoryId?: string;
  /** all | cny | jpy */
  currency?: string;
  search?: string;
}

export interface LedgerCreateCategory {
  name: string;
}

export interface LedgerCreateTransaction {
  amount: string;
  currency: string;
  direction: string;
  categoryId?: string | null;
  occurredOn: string;
  note?: string | null;
}

export interface LedgerUpdateTransaction {
  id: string;
  amount: string;
  currency: string;
  direction: string;
  categoryId?: string | null;
  occurredOn: string;
  note?: string | null;
}

export interface LedgerStatQuery {
  dateFrom: string;
  dateTo: string;
  direction: string;
  categoryId?: string | null;
  currency: string;
}

export interface DailySeriesRow {
  date: string;
  outSum: number;
  inSum: number;
}

export interface CategoryExpenseRow {
  name: string;
  total: number;
}

export interface LedgerStatistics {
  daily: DailySeriesRow[];
  categoryExpense: CategoryExpenseRow[];
}

export interface LedgerRollupCard {
  period: string;
  label: string;
  cnyIncome: string;
  cnyExpense: string;
  jpyIncome: string;
  jpyExpense: string;
}

export interface LedgerRollups {
  cards: LedgerRollupCard[];
}

export function ledgerListCategories(): Promise<LedgerCategoryDto[]> {
  return invoke('ledger_list_categories');
}

export function ledgerCreateCategory(
  payload: LedgerCreateCategory,
): Promise<LedgerCategoryDto> {
  return invoke('ledger_create_category', { payload });
}

export function ledgerDeleteCategory(id: string): Promise<void> {
  return invoke('ledger_delete_category', { id });
}

export function ledgerListTransactions(
  filter: LedgerListFilter,
): Promise<LedgerTransactionDto[]> {
  return invoke('ledger_list_transactions', { filter });
}

export function ledgerGetTransaction(
  id: string,
): Promise<LedgerTransactionDto | null> {
  return invoke('ledger_get_transaction', { id });
}

export function ledgerCreateTransaction(
  payload: LedgerCreateTransaction,
): Promise<LedgerTransactionDto> {
  return invoke('ledger_create_transaction', { payload });
}

export function ledgerUpdateTransaction(
  payload: LedgerUpdateTransaction,
): Promise<LedgerTransactionDto> {
  return invoke('ledger_update_transaction', { payload });
}

export function ledgerDeleteTransactions(ids: string[]): Promise<number> {
  return invoke('ledger_delete_transactions', { ids });
}

export function ledgerGetPeriodRollups(): Promise<LedgerRollups> {
  return invoke('ledger_get_period_rollups');
}

export function ledgerGetStatistics(
  query: LedgerStatQuery,
): Promise<LedgerStatistics> {
  return invoke('ledger_get_statistics', { query });
}
