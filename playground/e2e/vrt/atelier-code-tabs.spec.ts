import { expect, test, type Page } from '@playwright/test'

async function waitForAtelier(page: Page) {
  await page.goto('/?tab=atelier')
  await page.waitForFunction(
    () => document.querySelector('.wasm-status')?.textContent?.includes('WASM'),
    { timeout: 15_000 },
  )
  await page.waitForSelector('.compile-time', { timeout: 10_000 })
  await expect(page.locator('.code-output .code-content').first()).toBeVisible()
}

async function getCodeText(page: Page) {
  return (await page.locator('.code-output .code-content').first().textContent()) ?? ''
}

async function getCodeLines(page: Page) {
  return await page
    .locator('.code-output .code-line')
    .evaluateAll((nodes) => nodes.map((node) => node.textContent ?? ''))
}

async function waitForHighlightedOutput(page: Page) {
  await page.waitForFunction(
    () => !!document.querySelector('.code-output .code-line span[style*="--d:"]'),
    { timeout: 10_000 },
  )
}

async function getFirstHighlightedTokenStyle(page: Page) {
  return await page.evaluate(() => {
    const token = document.querySelector('.code-output .code-line span[style*="--d:"]')
    const content = document.querySelector('.code-output .code-content')
    if (!(token instanceof HTMLElement) || !(content instanceof HTMLElement)) {
      return null
    }
    return {
      text: token.textContent ?? '',
      color: getComputedStyle(token).color,
      containerColor: getComputedStyle(content).color,
    }
  })
}

test('atelier code targets expose VDOM, SSR, and Vapor outputs with stable toggles', async ({ page }) => {
  await waitForAtelier(page)
  await waitForHighlightedOutput(page)

  const initialTokenStyle = await getFirstHighlightedTokenStyle(page)
  expect(initialTokenStyle).toEqual({
    text: 'import',
    color: 'rgb(115, 96, 62)',
    containerColor: 'rgb(18, 18, 18)',
  })

  const vdomButton = page.getByRole('button', { name: 'VDOM' })
  const ssrButton = page.getByRole('button', { name: 'SSR' })
  const vaporButton = page.getByRole('button', { name: 'Vapor' })
  const vaporSsrButton = page.getByRole('button', { name: 'Vapor SSR' })
  const tsButton = page.locator('.code-view-toggle').getByRole('button', { name: 'TS' })
  const jsButton = page.locator('.code-view-toggle').getByRole('button', { name: 'JS' })

  await expect(vdomButton).toBeVisible()
  await expect(ssrButton).toBeVisible()
  await expect(vaporButton).toBeVisible()
  await expect(vaporSsrButton).toHaveCount(0)
  await expect(tsButton).toBeEnabled()
  await expect(jsButton).toBeEnabled()

  await ssrButton.click()
  await expect(page.locator('.code-header h4')).toHaveText('SSR Output')
  await expect(tsButton).toBeDisabled()
  await expect(jsButton).toBeDisabled()
  await expect(page.getByText('SFC Output')).toHaveCount(0)
  await waitForHighlightedOutput(page)
  await expect.poll(() => getFirstHighlightedTokenStyle(page)).toEqual({
    text: 'import',
    color: 'rgb(115, 96, 62)',
    containerColor: 'rgb(18, 18, 18)',
  })

  const ssrCode = await getCodeText(page)
  const ssrLines = await getCodeLines(page)
  expect(ssrCode).toContain('ssrRender')
  expect(ssrCode).not.toContain('_ctx.name')
  expect(ssrCode).not.toContain('_ctx.doubled')
  expect(ssrLines.some((line) => line.includes('<div class="card">'))).toBe(true)
  expect(ssrLines).toContain('  <h2>${_ssrInterpolate(name)}</h2>')
  expect(ssrLines).toContain("  <button${_ssrRenderAttr('disabled', disabled)}>")
  expect(ssrLines).toContain('      Increment')
  expect(ssrLines).toContain('    </button>')

  await vaporButton.click()
  await expect(page.locator('.code-header h4')).toHaveText('Vapor Output')
  await expect(tsButton).toBeDisabled()
  await expect(jsButton).toBeDisabled()
  await expect(page.getByText('Template Fragments')).toHaveCount(0)
  await expect(page.getByText('SFC Output')).toHaveCount(0)
  await waitForHighlightedOutput(page)
  await expect.poll(() => getFirstHighlightedTokenStyle(page)).toEqual({
    text: 'import',
    color: 'rgb(115, 96, 62)',
    containerColor: 'rgb(18, 18, 18)',
  })

  const vaporCode = await getCodeText(page)
  const vaporLines = await getCodeLines(page)
  expect(vaporCode).toContain('const t0 = _template')
  expect(vaporCode).toContain('_renderEffect')
  expect(vaporLines.some((line) => line.includes('<div class="card">'))).toBe(true)
  expect(vaporLines.some((line) => line.includes('<h2> </h2>'))).toBe(true)
  expect(vaporLines.some((line) => line.includes('Increment'))).toBe(true)
})
