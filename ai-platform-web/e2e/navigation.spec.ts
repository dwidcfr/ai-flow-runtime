import { test, expect } from '@playwright/test'

test('home page shows Projects heading', async ({ page }) => {
  await page.goto('/')
  await expect(page.getByRole('heading', { name: 'Projects' })).toBeVisible()
})

test('navigation to settings works', async ({ page }) => {
  await page.goto('/')
  await page.getByText('Settings').click()
  await expect(page).toHaveURL(/\/settings/)
  await expect(page.getByRole('heading', { name: 'Settings' })).toBeVisible()
})
