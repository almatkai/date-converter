"""Produces an UNAPPLIED generic host patch; never reads the app working files."""
import pathlib,difflib,shutil
base=pathlib.Path(__file__).parent/'host';out=pathlib.Path(__file__).parents[1]/'.qa/host-after'
shutil.copytree(base,out,dirs_exist_ok=True)
def edit(p,old,new):
 f=out/p;s=f.read_text();assert old in s,(p,old);f.write_text(s.replace(old,new))
reg='src-tauri/src/native_core/extensions/registry.rs'
# Retain canonical preference/uninstall path, keep the verified active resource path separate.
edit(reg,'    path: PathBuf,\n    manifest:','    path: PathBuf,\n    resource_path: PathBuf,\n    manifest:')
edit(reg,'            let mut manifest = if native {','            let mut resource_path = path.clone();\n            let mut manifest = if native {')
edit(reg,'Ok(package) => package.manifest,','Ok(package) => { resource_path = package.dir; package.manifest },')
edit(reg,'                path,\n                manifest,','                path,\n                resource_path,\n                manifest,')
edit(reg,'        path: root.clone(),','        path: root.clone(),\n        resource_path: root.clone(),') if '        path: root.clone(),' in (out/reg).read_text() else None
edit(reg,'        path,\n        manifest: Value::Null,','        resource_path: path.clone(),\n        path,\n        manifest: Value::Null,')
# All direct Installed struct constructions must be reviewed by the parent on application.
s=(out/reg).read_text();start=s.index('fn icon_path(');end=s.index('fn command_icon_data(',start)
s=s[:start]+s[start:end].replace('extension.path.join(', 'extension.resource_path.join(')+s[end:];(out/reg).write_text(s)
# Verified aliases are exact, case-sensitive, command-covered and ambiguity rejecting.
alias='''pub(super) fn verified_alias(ctx: &CoreContext, requested: &str, command: &str) -> Result<Option<String>, String> {
    if requested.starts_with("com.tezbar.native.") || requested.trim() != requested { return Ok(None); }
    let mut matches = Vec::new();
    for e in collect(ctx) {
        if !e.id.starts_with("com.tezbar.native.") || !e.manifest["healthError"].is_null() { continue; }
        if !e.manifest["replaces"].as_array().is_some_and(|a|a.iter().any(|v|v.as_str()==Some(requested))) { continue; }
        let Ok(package) = super::process_install::resolve(ctx, &e.id) else { continue; };
        if !package.manifest["replaces"].as_array().is_some_and(|a|a.iter().any(|v|v.as_str()==Some(requested))) { continue; }
        let covered = package.manifest["commands"].as_array().is_some_and(|a|a.iter().any(|c|c["name"]==command && c["runtime"]=="rust" && compatible(c,true)));
        matches.push((e.id,covered));
    }
    if matches.len()>1 { return Err("Ambiguous verified extension alias".into()); }
    Ok(matches.pop().and_then(|(id,covered)|covered.then_some(id)))
}

'''
edit(reg,'pub(super) fn installed_command(',alias+'pub(super) fn installed_command(')
# installed_command canonical is invoked by native.launch after alias resolution.
native='src-tauri/src/native_core/extensions/native.rs'
edit(native,'if requested.starts_with("com.tezbar.native.") || v2_runtime::slug(requested).is_some()', 'if requested.starts_with("com.tezbar.native.") || v2_runtime::slug(requested).is_some() || match registry::verified_alias(ctx, requested, body["commandName"].as_str().unwrap_or("")) { Ok(v) => v.is_some(), Err(e) => return Some(Ok(json!({"ok":false,"message":e}))) }')
edit(native,'        } else {\n            v2_runtime::canonical_id(v2_runtime::slug(requested).ok_or("Unknown extension")?)\n        };','        } else if let Some(id) = registry::verified_alias(ctx, requested, command)? {\n            id\n        } else {\n            v2_runtime::canonical_id(v2_runtime::slug(requested).ok_or("Unknown extension")?)\n        };')
edit('src/renderer/src/raycast-api/list-runtime.tsx', '  onLoadMore,\n}: {', '  onLoadMore,\n  onDraftChange,\n}: {')
edit('src/renderer/src/raycast-api/list-runtime.tsx', '  onLoadMore: () => Promise<void> | void\n', '  onLoadMore: () => Promise<void> | void\n  onDraftChange?: (value: { searchText: string; dirty: boolean; searchDirty: boolean; accessoryValue: string; token: unknown }) => void\n')
edit('src/renderer/src/raycast-api/list-runtime.tsx', 'type ListAccessory = {\n  actionId?: string', 'type ListAccessory = {\n  value?: string\n  actionId?: string')
edit('src/renderer/src/raycast-api/list-runtime.tsx', '    actionId: typeof node.props?.actionId', "    value: typeof node.props?.value === 'string' ? node.props.value : undefined,\n    actionId: typeof node.props?.actionId")
edit('src/renderer/src/raycast-api/list-runtime.tsx', '  const [selected, setSelected] = useState(0)', "  const [draftRevision, setDraftRevision] = useState(0)\n  const draftDirty = useRef(false)\n  const draftText = useRef(query)\n  const accessoryDraft = useRef(String(root.props?.__timezone ?? ''))\n  const accessoryDirty = useRef(false)\n  const [accessoryPending, setAccessoryPending] = useState(false)\n  const [selected, setSelected] = useState(0)")
edit('src/renderer/src/raycast-api/list-runtime.tsx', "    searchAccessory?.options[0]?.value ?? '',", "    searchAccessory?.value ?? searchAccessory?.options[0]?.value ?? '',")
edit('src/renderer/src/raycast-api/list-runtime.tsx', '      setQuery(root.props.searchText)', "      if (root.props?.__draftGuard && ((draftDirty.current && root.props.searchText !== draftText.current) || (accessoryDirty.current && root.props.__timezone !== accessoryDraft.current))) return\n      setQuery(root.props.searchText)\n      draftText.current = root.props.searchText\n      draftDirty.current = false\n      accessoryDirty.current = false\n      setAccessoryPending(false)\n      accessoryDraft.current = String(root.props?.__timezone ?? '')\n      onDraftChange?.({ searchText: root.props.searchText, dirty: false, searchDirty: false, accessoryValue: accessoryDraft.current, token: root.props?.__draftGuard })")
edit('src/renderer/src/raycast-api/list-runtime.tsx', '  }, [root.props?.searchText])', '  }, [root.props?.searchText, root.props?.__draftGuard])')
edit('src/renderer/src/raycast-api/list-runtime.tsx', '      return searchAccessory.options.some', '      if (searchAccessory.value !== undefined) return searchAccessory.value\n      return searchAccessory.options.some')
edit('src/renderer/src/raycast-api/list-runtime.tsx', '    if (query === lastSentQuery.current) return', '    if (query === lastSentQuery.current && !draftDirty.current) return')
edit('src/renderer/src/raycast-api/list-runtime.tsx', '  }, [query, onSearchTextChanged])', '  }, [query, onSearchTextChanged, draftRevision])')
edit('src/renderer/src/raycast-api/list-runtime.tsx', 'onChange={(event) => {\n                  setQuery(event.target.value)', 'onChange={(event) => {\n                  const text = event.target.value\n                  draftText.current = text\n                  draftDirty.current = true\n                  onDraftChange?.({ searchText: text, dirty: true, searchDirty: true, accessoryValue: accessoryDraft.current, token: root.props?.__draftGuard })\n                  setDraftRevision((value) => value + 1)\n                  setQuery(text)')
edit('src/renderer/src/raycast-api/list-runtime.tsx', '                        setAccessoryValue(option.value)', '                        if (accessoryDirty.current) return\n                        accessoryDraft.current = option.value\n                        accessoryDirty.current = true\n                        setAccessoryPending(true)\n                        onDraftChange?.({ searchText: draftText.current, dirty: true, searchDirty: draftDirty.current, accessoryValue: option.value, token: root.props?.__draftGuard })\n                        setAccessoryValue(option.value)')
edit('src/renderer/src/raycast-api/list-runtime.tsx', '                      key={option.value}\n                      type="button"', '                      key={option.value}\n                      disabled={accessoryPending}\n                      type="button"')
edit('src/renderer/src/raycast-api/list-runtime.tsx', "    : hasQuery\n      ? 'No matching applications'", "    : hasQuery && !hasServerSearch\n      ? 'No matching applications'")
edit('src/renderer/src/raycast-api/list-runtime.tsx', "    : hasQuery\n      ? 'Try a different app name or keyword.'", "    : hasQuery && !hasServerSearch\n      ? 'Try a different app name or keyword.'")
edit('src/renderer/src/raycast-api/index.tsx', '  onInvokeAction: (actionId:', '  onInputDraftChange?: () => void\n  onInvokeAction: (actionId:')
edit('src/renderer/src/raycast-api/index.tsx', '    onInvokeAction,\n  } = props', '    onInvokeAction: invokeHost,\n    onInputDraftChange,\n  } = props\n  const listDraft = useRef<{ searchText: string; dirty: boolean; searchDirty: boolean; accessoryValue: string; token: unknown } | null>(null)\n  const onInvokeAction = useCallback((id: string, values?: Record<string, unknown>) => {\n    const draft = listDraft.current\n    return invokeHost(id, root.props?.__draftGuard ? { ...values, __searchText: draft?.searchText ?? root.props.searchText, __draftDirty: draft?.dirty ?? false, __searchDirty: draft?.searchDirty ?? false, __accessoryValue: draft?.accessoryValue ?? root.props.__timezone, __viewToken: draft?.token ?? root.props.__draftGuard } : values)\n  }, [invokeHost, root])')
edit('src/renderer/src/raycast-api/index.tsx', '            <ListRuntime\n', '            <ListRuntime\n              onDraftChange={(draft) => { listDraft.current = draft; if (draft.dirty) onInputDraftChange?.() }}\n')
edit('src/renderer/ExtensionRuntimeView.tsx', '  const searchRequestSeq = useRef(0)', '  const searchRequestSeq = useRef(0)\n  const inputRevision = useRef(0)')
edit('src/renderer/ExtensionRuntimeView.tsx', '            onInvokeAction={async (actionId, formValues) => {', '            onInputDraftChange={() => { inputRevision.current += 1; searchRequestSeq.current += 1 }}\n            onInvokeAction={async (actionId, formValues) => {\n              const inputOwner = inputRevision.current')
edit('src/renderer/ExtensionRuntimeView.tsx', '              if (currentSession.current !== sessionOwner) return\n              if (!result.ok)', '              if (currentSession.current !== sessionOwner || inputRevision.current !== inputOwner) return\n              if (!result.ok)')

s=(out/'src/renderer/src/raycast-api/list-runtime.tsx').read_text()
start=s.index('                {searchAccessory.options.map((option) => {')
end=s.index('\n              </div>',start)
old=s[start:end]
select='<select\n                  aria-label="Select Target Timezone"\n                  value={accessoryValue}\n                  disabled={accessoryPending}\n                  className="h-8 w-full rounded-tezbar-chip border border-white/10 bg-panel px-2.5 text-[12px] text-ink-1"\n                  onChange={(event) => {\n                    if (accessoryDirty.current) return\n                    const value = event.target.value\n                    accessoryDraft.current = value\n                    accessoryDirty.current = true\n                    setAccessoryPending(true)\n                    onDraftChange?.({ searchText: draftText.current, dirty: true, searchDirty: draftDirty.current, accessoryValue: value, token: root.props?.__draftGuard })\n                    setAccessoryValue(value)\n                    setSelected(0)\n                    if (searchAccessory.actionId) onRunPrimaryAction(searchAccessory.actionId, { value })\n                  }}\n                >\n                  {searchAccessory.options.map((option) => <option key={option.value} value={option.value}>{option.title}</option>)}\n                </select>'
s=s[:start]+old.replace('{searchAccessory.options.map', '{hasManySearchOptions ? '+select+' : searchAccessory.options.map')+s[end:]
(out/'src/renderer/src/raycast-api/list-runtime.tsx').write_text(s)
edit('src/renderer/src/raycast-api/list-runtime.tsx', '      if (searchAccessory.value !== undefined) return searchAccessory.value', "      if (root.props?.__draftGuard && accessoryDirty.current && searchAccessory.value !== accessoryDraft.current) return current\n      if (searchAccessory.value !== undefined) return searchAccessory.value")
patch=[]
for f in sorted(out.rglob('*')):
 if f.is_file():
  rel=f.relative_to(out);old=(base/rel).read_text();new=f.read_text()
  if old!=new:patch.extend(difflib.unified_diff(old.splitlines(True),new.splitlines(True),fromfile='a/'+str(rel),tofile='b/'+str(rel)))
dest=pathlib.Path(__file__).parents[1]/'integration';dest.mkdir(exist_ok=True);(dest/'host-prerequisites.patch').write_text(''.join(patch))
print('Generated unapplied patch', len(''.join(patch)), 'bytes')
