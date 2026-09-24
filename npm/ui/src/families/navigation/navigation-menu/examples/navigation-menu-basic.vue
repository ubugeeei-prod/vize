<!-- Site navigation with two flyout sections, a direct link, an indicator, and a shared viewport. -->
<script setup lang="ts">
import {
  NavigationMenuContent,
  NavigationMenuIndicator,
  NavigationMenuItem,
  NavigationMenuLink,
  NavigationMenuList,
  NavigationMenuRoot,
  NavigationMenuTrigger,
  NavigationMenuViewport,
} from "../navigation-menu.ts";
</script>

<template>
  <NavigationMenuRoot aria-label="Main">
    <NavigationMenuList>
      <NavigationMenuItem value="products">
        <NavigationMenuTrigger>Products</NavigationMenuTrigger>
        <NavigationMenuContent>
          <ul>
            <li><NavigationMenuLink href="/products/analytics">Analytics</NavigationMenuLink></li>
            <li><NavigationMenuLink href="/products/automation">Automation</NavigationMenuLink></li>
          </ul>
        </NavigationMenuContent>
      </NavigationMenuItem>
      <NavigationMenuItem value="resources">
        <NavigationMenuTrigger>Resources</NavigationMenuTrigger>
        <NavigationMenuContent>
          <ul>
            <li><NavigationMenuLink href="/docs">Documentation</NavigationMenuLink></li>
            <li><NavigationMenuLink href="/blog">Blog</NavigationMenuLink></li>
          </ul>
        </NavigationMenuContent>
      </NavigationMenuItem>
      <NavigationMenuItem value="pricing">
        <NavigationMenuLink href="/pricing" active>Pricing</NavigationMenuLink>
      </NavigationMenuItem>
      <NavigationMenuIndicator />
    </NavigationMenuList>
    <NavigationMenuViewport />
  </NavigationMenuRoot>
</template>
