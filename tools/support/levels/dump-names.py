"""Explicit nominal API map for the mechanical dump migration."""
import re

BASE = '1831d54dcf739f78afac96e06eb62159f44bbb4b'
SPECIAL = {
    'Folio': 'Dump', 'FolioMode': 'DumpMode', 'FolioError': 'DumpError',
    'FolioValue': 'DumpValue', 'FolioDump': 'Collector', 'DumpPage': 'CollectedPage',
    'FusionPlanFolio': 'PlanPage', 'FolioPlanPass': 'PlanPass',
    'CroquisFolio': 'CroquisPage', 'ReproFolio': 'ReproPage',
    'L2Folio': 'L2Page', 'DisegnoFolio': 'L2Page',
    'L3Folio': 'L3Page', 'ImpetoFolio': 'L3Page',
    'L2ProvenanceFolio': 'ProvenancePage', 'FolioProvenance': 'DumpProvenance',
    'L3ExtractionFolio': 'ExtractionPage', 'L3PlacementFolio': 'PlacementPage',
    'L3ReactivityFolio': 'ReactivityPage', 'ReactivityFolio': 'ReactivityPage',
    'L3PartitionFolio': 'PartitionPage', 'L3ValuesFolio': 'ValuesPage',
    'FolioObserver': 'DumpObserver', 'assert_folio_snapshot': 'assert_dump_snapshot',
    'derive_folio': 'derive_dump', 'FolioContains': 'FolioContains',
}
CONCERNS = {
    'vize_davinci::folio': {'Folio': 'Dump', 'FolioMode': 'Mode', 'FolioError': 'Error'},
    'vize_davinci::folio::value': {'FolioValue': 'DumpValue'},
    'vize_davinci::folio::dump': {'FolioDump': 'Collector', 'DumpPage': 'Page'},
    'vize_davinci::folio::plan': {'FusionPlanFolio': 'Page', 'FolioPlanPass': 'Pass'},
    'vize_davinci::folio::croquis': {'CroquisFolio': 'Page'},
    'vize_davinci::folio::repro': {'ReproFolio': 'Page'},
    'vize_l2::folio': {'L2Folio': 'Page', 'DisegnoFolio': 'Page'},
    'vize_l2::folio::provenance': {'L2ProvenanceFolio': 'Page', 'FolioProvenance': 'Record'},
    'vize_l3::folio': {'L3Folio': 'Page', 'ImpetoFolio': 'Page'},
    'vize_l3::extract::folio': {'L3ExtractionFolio': 'Page', 'FolioDecision': 'Record'},
    'vize_l3::placement::folio': {'L3PlacementFolio': 'Page', 'FolioPlacement': 'Record'},
    'vize_l3::lattice::folio': {'L3ReactivityFolio': 'Page', 'ReactivityFolio': 'Page', 'FolioBinding': 'Binding'},
    'vize_l3::values_folio': {'L3ValuesFolio': 'Page', 'FolioOperand': 'Operand'},
    'vize_l2_to_l3::partition::folio': {'L3PartitionFolio': 'Page', 'FolioPartitionFact': 'Fact'},
}


def generic(identifier):
    return SPECIAL.get(identifier, re.sub(r'^Folio', 'Dump', identifier))


def module(path):
    parts = path.split('/')
    if len(parts) < 4 or parts[0] != 'crates' or parts[2] != 'src':
        return None
    tail = '/'.join(parts[3:]).removesuffix('.rs')
    return parts[1] + ('' if tail == 'lib' else '::' + tail.replace('/', '::'))


def destination(path):
    if path == 'crates/vize_davinci/src/folio/dump.rs':
        return 'crates/vize_davinci/src/dump/collector.rs'
    return path.replace('/folio/', '/dump/').replace('/folio.rs', '/dump.rs').replace('_folio.rs', '_dump.rs')


def namespace(path):
    return path.replace('::folio::dump', '::dump::collector').replace('::folio', '::dump').replace('::values_folio', '::values_dump')


def local(path, identifier):
    if path == 'crates/vize_l2/tests/folio_laws.rs':
        if identifier in {'L2Folio', 'DisegnoFolio'}:
            return 'Page'
        if identifier.startswith('Folio') and identifier not in SPECIAL:
            return identifier[5:]
    owner = module(path)
    if owner:
        for concern, names in CONCERNS.items():
            if owner == concern or (concern in {'vize_l2::folio', 'vize_davinci::folio::croquis'} and owner.startswith(concern + '::')):
                if identifier in names:
                    return names[identifier]
        if owner.startswith('vize_l2::folio') or owner == 'vize_l3::folio':
            if identifier.startswith('Folio') and identifier not in SPECIAL:
                return identifier[5:]
    return generic(identifier)


def targets(sources):
    result = {'vize_davinci_derive::Folio': 'vize_davinci_derive::Dump'}
    for path, text in sources.items():
        owner = module(path)
        if not owner:
            continue
        seen = {}
        for identifier in re.findall(r'pub (?:struct|enum|trait|type) (\w+)', text):
            replacement = local(path, identifier)
            if replacement in seen and identifier not in {'DisegnoFolio', 'ImpetoFolio', 'ReactivityFolio'}:
                raise ValueError(f'{path}: colliding declaration {seen[replacement]} / {identifier} -> {replacement}')
            seen[replacement] = identifier
            if replacement != identifier:
                result[owner + '::' + identifier] = namespace(owner) + '::' + replacement
    # Former re-export paths map to the same canonical defining namespace.
    for owner, names in CONCERNS.items():
        for identifier, replacement in names.items():
            result[owner + '::' + identifier] = namespace(owner) + '::' + replacement
    for identifier in ('L2ProvenanceFolio', 'FolioProvenance'):
        result['vize_l2::folio::' + identifier] = result['vize_l2::folio::provenance::' + identifier]
    for owner in ('extract', 'placement', 'lattice'):
        for identifier in CONCERNS['vize_l3::' + owner + '::folio']:
            result['vize_l3::' + owner + '::' + identifier] = result['vize_l3::' + owner + '::folio::' + identifier]
    for identifier in ('L3PartitionFolio', 'FolioPartitionFact'):
        target = result['vize_l2_to_l3::partition::folio::' + identifier]
        result['vize_l2_to_l3::partition::' + identifier] = target
        result['vize_l2_to_l3::' + identifier] = target
    for identifier, target in list(result.items()):
        if identifier.startswith('vize_l2::folio::owned::') and '::binding::' not in identifier and '::expr::' not in identifier:
            result['vize_l2::folio::' + identifier.rsplit('::', 1)[1]] = 'vize_l2::dump::' + target.rsplit('::', 1)[1]
        elif identifier.startswith('vize_l2::folio::owned::'):
            result['vize_l2::folio::' + identifier.rsplit('::', 1)[1]] = 'vize_l2::dump::' + target.rsplit('::', 1)[1]
    return result
