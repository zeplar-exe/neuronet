#!/usr/bin/env python3
"""Generates a typed Python ctypes wrapper from neuronet.h using pycparser."""

import os

from pycparser import c_ast, parse_file

C_TO_CTYPES = {
    "void": "None",
    "bool": "c_bool",
    "_Bool": "c_bool",
    "int": "c_int",
    "uint8_t": "c_ubyte",
    "int32_t": "c_int",
    "uint32_t": "c_uint",
    "uintptr_t": "c_size_t",
    "double": "c_double",
    "float": "c_float",
    "unsigned char": "c_ubyte",
    "char": "c_ubyte",
    "unsigned int": "c_uint",
    "unsigned long": "c_size_t",
}
C_TO_PYTHON = {
    "None": "None",
    "c_bool": "bool",
    "c_ubyte": "int",
    "c_int": "int",
    "c_uint": "int",
    "c_size_t": "int",
    "c_double": "float",
    "c_float": "float",
    "c_void_p": "int",
}


class Collector(c_ast.NodeVisitor):
    def __init__(self):
        self.typedefs = {}
        self.enums = {}
        self.structs = {}
        self.opaque = set()
        self.funcs = []

    def visit_Typedef(self, node):
        name = node.name
        typ = node.type

        if isinstance(typ, c_ast.TypeDecl):
            inner = typ.type
            if isinstance(inner, c_ast.Enum):
                variants = (
                    [e.name for e in inner.values.enumerators] if inner.values else []
                )
                self.enums[name] = variants
            elif isinstance(inner, c_ast.Struct):
                if inner.decls is None:
                    self.opaque.add(name)
                else:
                    self.structs[name] = self._collect_fields(inner.decls)
            elif isinstance(inner, c_ast.IdentifierType):
                self.typedefs[name] = " ".join(inner.names)
        elif isinstance(typ, c_ast.PtrDecl):
            pass

    def _collect_fields(self, decls):
        fields = []
        for d in decls:
            fname = d.name
            ftype = self._resolve_type(d.type)
            fields.append((fname, ftype))
        return fields

    def _resolve_type(self, node):
        if isinstance(node, c_ast.TypeDecl):
            return self._resolve_type(node.type)
        elif isinstance(node, c_ast.IdentifierType):
            return " ".join(node.names)
        elif isinstance(node, c_ast.PtrDecl):
            inner = self._resolve_type(node.type)
            return inner + " *"
        elif isinstance(node, c_ast.Struct):
            return node.name or "void"
        elif isinstance(node, c_ast.Enum):
            return node.name or "int"
        elif isinstance(node, c_ast.ArrayDecl):
            return self._resolve_type(node.type) + "[]"
        return "void"

    def visit_FuncDecl(self, node):
        pass

    def visit_Decl(self, node):
        if not isinstance(node.type, c_ast.FuncDecl):
            return
        fdecl = node.type
        name = node.name
        ret = self._resolve_type(fdecl.type)

        params = []
        if fdecl.args:
            for p in fdecl.args.params:
                if isinstance(p, c_ast.EllipsisParam):
                    continue
                if isinstance(p, c_ast.Typename):
                    ptype = self._resolve_type(p.type)
                    if ptype == "void":
                        continue
                    params.append(("arg", ptype))
                else:
                    ptype = self._resolve_type(p.type)
                    pname = p.name or "arg"
                    params.append((pname, ptype))

        self.funcs.append((name, ret, params))


def to_ctype(c_type, typedefs, structs, opaque, enums):
    clean = c_type.replace("const ", "").strip()

    # Resolve typedefs
    seen = set()
    r = clean
    while r in typedefs and r not in seen:
        seen.add(r)
        r = typedefs[r]

    if r.endswith(" *"):
        pointee = r[:-2].strip()
        seen2 = set()
        while pointee in typedefs and pointee not in seen2:
            seen2.add(pointee)
            pointee = typedefs[pointee]
        if pointee in opaque:
            return "c_void_p"
        if pointee in structs:
            return f"POINTER({pointee})"
        base = C_TO_CTYPES.get(pointee)
        return f"POINTER({base})" if base else "c_void_p"

    if r in C_TO_CTYPES:
        return C_TO_CTYPES[r]
    if r in structs:
        return r
    if r in enums:
        return "c_uint"
    return "c_void_p"


def pytype(ct):
    return C_TO_PYTHON.get(ct, ct)


FAKE_PREAMBLE = """
typedef unsigned char uint8_t;
typedef int int32_t;
typedef unsigned int uint32_t;
typedef unsigned long uintptr_t;
typedef _Bool bool;
"""


def generate(header_path, output_path, lib_name):
    with open(header_path) as f:
        src = f.read()
    lines = []
    for line in src.splitlines():
        stripped = line.strip()
        if stripped.startswith("#"):
            continue
        lines.append(line)
    from pycparser import CParser

    parser = CParser()
    ast = parser.parse(FAKE_PREAMBLE + "\n".join(lines), filename=header_path)
    c = Collector()
    c.visit(ast)

    R = lambda t: to_ctype(t, c.typedefs, c.structs, c.opaque, c.enums)
    out = []
    out.append("import ctypes")
    out.append("import os")
    out.append(
        "from ctypes import c_void_p, c_uint, c_int, c_double, c_size_t, c_bool, c_ubyte, POINTER, Structure"
    )
    out.append("from enum import IntEnum")
    out.append("")
    out.append(
        f'_lib_path = os.path.join(os.path.dirname(__file__), "..", "sim-backend", "target", "release", "lib{lib_name}.dylib")'
    )
    out.append(f"_lib = ctypes.CDLL(_lib_path)")
    out.append("")

    for name, variants in c.enums.items():
        out.append(f"class {name}(IntEnum):")
        for i, v in enumerate(variants):
            out.append(f"    {v} = {i}")
        out.append("")

    for name, fields in c.structs.items():
        out.append(f"class {name}(Structure):")
        for fname, ftype in fields:
            out.append(f"    {fname}: {R(ftype)}")
        out.append("    _fields_ = [")
        for fname, ftype in fields:
            out.append(f'        ("{fname}", {R(ftype)}),')
        out.append("    ]")
        out.append("")

    out.append("# --- ctypes setup ---")
    for fname, ret, params in c.funcs:
        argtypes = [R(pt) for _, pt in params]
        rettype = R(ret)
        parts = []
        if argtypes:
            parts.append(f"_lib.{fname}.argtypes = [{', '.join(argtypes)}]")
        if rettype != "None":
            parts.append(f"_lib.{fname}.restype = {rettype}")
        if parts:
            out.append("; ".join(parts))

    out.append("")
    out.append("# --- Public API ---")
    for fname, ret, params in c.funcs:
        rettype = R(ret)
        ret_clean = ret.replace("const ", "").replace(" *", "").strip()
        seen = set()
        while ret_clean in c.typedefs and ret_clean not in seen:
            seen.add(ret_clean)
            ret_clean = c.typedefs[ret_clean]

        py_params = ", ".join(f"{pn}: {pytype(R(pt))}" for pn, pt in params)
        call_args = ", ".join(pn for pn, _ in params)

        if rettype == "None":
            out.append(f"def {fname}({py_params}) -> None: _lib.{fname}({call_args})")
        elif ret_clean in c.enums:
            out.append(
                f"def {fname}({py_params}) -> {ret_clean}: return {ret_clean}(_lib.{fname}({call_args}))"
            )
        else:
            out.append(
                f"def {fname}({py_params}) -> {pytype(rettype)}: return _lib.{fname}({call_args})"
            )

    BUFFER_HELPERS = {
        "NeuronBuffer": ("get_neurons", "destroy_neuron_buffer", "iter_neurons", "int"),
        "SynapseBuffer": ("get_synapses", "destroy_synapse_buffer", "iter_synapses", "int"),
    }
    out.append("")
    out.append("# --- Iterators ---")
    for buf_type, (get_fn, destroy_fn, helper_name, item_type) in BUFFER_HELPERS.items():
        if buf_type in c.structs:
            out.append(f"def {helper_name}(network: int) -> list[{item_type}]:")
            out.append(f"    buf = _lib.{get_fn}(network)")
            out.append(f"    result = [buf.data[i] for i in range(buf.len)]")
            out.append(f"    _lib.{destroy_fn}(buf)")
            out.append(f"    return result")
            out.append("")

    out.append("")
    with open(output_path, "w") as f:
        f.write("\n".join(out))
    print(f"Generated {output_path}")


if __name__ == "__main__":
    script_dir = os.path.dirname(os.path.abspath(__file__))
    generate(
        os.path.join(script_dir, "neuronet.h"),
        os.path.join(script_dir, "..", "discretization-test", "neuronet.py"),
        "neuronet",
    )
    generate(
        os.path.join(script_dir, "neuronet.h"),
        os.path.join(script_dir, "..", "cyclic-genetics", "neuronet.py"),
        "neuronet",
    )
