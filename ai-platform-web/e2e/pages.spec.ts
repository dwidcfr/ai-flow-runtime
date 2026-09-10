import { test, expect } from '@playwright/test'

test('playground page loads', async ({ page }) => {
  await page.goto('/playground')
  await expect(page.getByRole('heading', { name: 'Playground' })).toBeVisible()
  await expect(page.getByLabel('Flow ID')).toBeVisible()
})

test('monitoring page shows overview cards', async ({ page }) => {
  await page.goto('/monitoring')
  await expect(page.getByRole('heading', { name: 'Monitoring' })).toBeVisible()
  await expect(page.getByText('Platform Status')).toBeVisible()
})
