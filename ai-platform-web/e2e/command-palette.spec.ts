import { test, expect } from '@playwright/test'

test('command palette opens with Ctrl+K', async ({ page }) => {
  await page.goto('/')
  await page.getByText('Ctrl+K to navigate').click()
  await expect(page.getByRole('dialog')).toBeVisible()
  await expect(page.getByRole('textbox').first()).toBeVisible()
})
