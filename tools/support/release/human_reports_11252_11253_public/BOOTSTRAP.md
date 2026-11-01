The public report workflow prepares the reporter's pnpm 12.16.1 through the
pinned setup action and this directory's `pnpm-bootstrap.json`. The action's
`package_json_file` input selects this isolated manifest; the repository's own
package manager declaration belongs to repository development.

The bootstrap manifest is included in both local and remote controller custody.
It is separate from each original consumer's package.json. Product installation
still invokes the verified pnpm executable with the original `install` argv,
without extra install flags or a replacement consumer manifest.

Before captures retain observations and have no product or issue qualification
credit. A setup failure occurs before those observations. Preserve that failure
and its original logs separately from any later capture.
